use crate::commands::create::handle_create;
use crate::commands::subscribe::handle_subscribe;
use crate::commands::unsubscribe::handle_unsubscribe;
use crate::commands::user::{handle_user, handle_users};
use crate::commands::{
    handle_info, handle_list, handle_login, handle_messages, handle_send, handle_subscribed_teams,
    handle_subscribed_users, handle_use,
};
use myteams::common::Message;
use myteams::common::StatusCode;
use myteams::common::client::Client;
use myteams::common::protocol::ResponseCode;
use myteams::common::protocol::command::Command;
use myteams::common::protocol::request::Request;
use myteams::common::user::User;
use myteams::common::utils::escape_str;
use myteams::common::utils::serializing::{read_length, write_length};
use myteams::common::{Response, Team};
use myteams::server_event_user_logged_out_safe;
use nix::poll::{PollFd, PollFlags};
use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use std::net::TcpListener;
use std::os::fd::AsFd;

//remember to change the load and save functions to persist state if something is added
pub struct Server {
    pub listener: TcpListener,
    pub clients: HashMap<String, Client>,
    pub users: HashMap<String, User>,
    pub private_messages: Vec<Message>,
    pub teams: HashMap<String, Team>,
}

impl Server {
    pub fn new() -> Server {
        // TODO: Implement load persistence
        Server {
            listener: TcpListener::bind("0.0.0.0:8080")
                .expect("Error starting server on port 8080"),
            clients: HashMap::new(),
            users: HashMap::new(),
            private_messages: Vec::new(),
            teams: HashMap::new(),
        }
    }

    pub fn run(&mut self) {
        let mut fds = vec![PollFd::new(
            self.listener.as_fd(),
            PollFlags::POLLIN | PollFlags::POLLOUT,
        )];
        let client_keys: Vec<String> = self.clients.keys().cloned().collect();

        for uuid in &client_keys {
            if let Some(client) = self.clients.get(uuid) {
                fds.push(PollFd::new(
                    client.socket.as_fd(),
                    PollFlags::POLLIN | PollFlags::POLLOUT,
                ));
            }
        }

        if let Err(_e) = nix::poll::poll(&mut fds, None::<u16>) {
            return;
        }

        let listener_ready = fds[0]
            .revents()
            .map_or(false, |f| f.contains(PollFlags::POLLIN));

        let client_revents: Vec<PollFlags> = fds[1..]
            .iter()
            .map(|fd| fd.revents().unwrap_or(PollFlags::empty()))
            .collect();

        drop(fds);

        if listener_ready {
            self.accept_connections();
        }

        self.process_client_events(client_revents, client_keys);
    }

    pub fn accept_connections(&mut self) {
        if let Ok((socket, _addr)) = self.listener.accept() {
            let new_client = Client::new(socket);
            self.clients.insert(new_client.uuid.clone(), new_client);
        }
    }

    pub fn process_client_events(
        &mut self,
        client_revents: Vec<PollFlags>,
        client_keys: Vec<String>,
    ) {
        let mut disconnected = Vec::new();
        let mut requests_to_read = Vec::new();
        let mut requests_to_write = Vec::new();

        for (i, uuid) in client_keys.into_iter().enumerate() {
            let revents = client_revents[i];

            if revents.contains(PollFlags::POLLIN) {
                if let Some(client) = self.clients.get_mut(&uuid) {
                    match client.read_data() {
                        Some(msg) => {
                            let trimmed = msg.trim();
                            if trimmed == "/disconnect" {
                                disconnected.push(uuid);
                                continue;
                            } else if trimmed == "LOGOUT" {
                                disconnected.push(uuid.clone());
                            } else {
                                match trimmed.parse::<Request>() {
                                    Ok(req) => {
                                        if client.user.is_none()
                                            && !matches!(req.command, Command::Login(_))
                                        {
                                            client.pending_responses.push(Response {
                                                code: ResponseCode::Status(
                                                    StatusCode::Unauthorized,
                                                ),
                                                data: None,
                                            });
                                            requests_to_write.push(uuid.clone());
                                            continue;
                                        }
                                        requests_to_read.push((uuid.clone(), req));
                                    }
                                    Err(_) => {
                                        if let Some(client) = self.clients.get_mut(&uuid) {
                                            client.pending_responses.push(Response {
                                                code: ResponseCode::Status(StatusCode::BadRequest),
                                                data: None,
                                            });
                                        }
                                    }
                                }
                            }
                        }
                        None => disconnected.push(uuid.clone()),
                    }
                }
            }
            if revents.contains(PollFlags::POLLOUT) {
                requests_to_write.push(uuid.clone())
            }
        }

        for (uuid, req) in requests_to_read {
            self.handle_request(&uuid, req);
        }

        for uuid in requests_to_write {
            let client = self.clients.get_mut(&uuid);
            match client {
                None => {}
                Some(c) => {
                    let responses: Vec<Response> = c.pending_responses.drain(..).collect();
                    for response in responses {
                        self.handle_response(uuid.as_str(), response);
                    }
                }
            }
        }

        for uuid in disconnected {
            if let Some(client) = self.clients.remove(&uuid) {
                if let Some(user_uuid) = &client.user {
                    server_event_user_logged_out_safe(user_uuid.as_str());

                    if let Some(user) = self.users.get(user_uuid) {
                        let event_resp = Response {
                            code: ResponseCode::Event(myteams::common::protocol::event::EventCode::LoggedOut),
                            data: Some(format!("\"{}\" \"{}\"", user.uuid, escape_str(&user.name))),
                        };
                        self.broadcast_global(event_resp);
                    }
                }
            }
        }
    }

    pub fn handle_request(&mut self, client_uuid: &str, request: Request) {
        match request.command {
            Command::Login(name) => handle_login(self, client_uuid, name),
            Command::Send(receiver_uuid, body) => {
                handle_send(self, client_uuid, receiver_uuid, body)
            }
            Command::Messages(other_user_uuid) => {
                handle_messages(self, client_uuid, other_user_uuid)
            }
            Command::Create(data) => handle_create(self, client_uuid, data),
            Command::Use(t, c, th) => handle_use(self, client_uuid, t, c, th),
            Command::List => handle_list(self, client_uuid),
            Command::Info => handle_info(self, client_uuid),
            Command::Users => handle_users(self, client_uuid),
            Command::User(user_uuid) => handle_user(self, client_uuid, user_uuid.as_str()),
            Command::Subscribe(team_uuid) => {
                handle_subscribe(self, client_uuid, team_uuid.as_str())
            }
            Command::Subscribed(team_uuid) => {
                if team_uuid.is_some() {
                    handle_subscribed_users(self, client_uuid, team_uuid.unwrap());
                } else {
                    handle_subscribed_teams(self, client_uuid);
                }
            }
            Command::Unsubscribe(team_uuid) => {
                handle_unsubscribe(self, client_uuid, team_uuid.as_str())
            }
            _ => {
                if let Some(client) = self.clients.get_mut(client_uuid) {
                    client.pending_responses.push(Response {
                        code: ResponseCode::Status(StatusCode::BadRequest),
                        data: None,
                    });
                }
            }
        }
    }

    pub fn handle_response(&mut self, client_uuid: &str, response: Response) {
        let client = self.clients.get_mut(client_uuid).unwrap();
        let _ = client.socket.write_all(response.to_string().as_ref());
    }

    pub fn broadcast_global(&mut self, event: Response) {
        for client in self.clients.values_mut() {
            if client.user.is_some() {
                client.pending_responses.push(event.clone());
            }
        }
    }

    pub fn broadcast_to_team(&mut self, team_members: &[String], event: Response) {
        for client in self.clients.values_mut() {
            if let Some(user_uuid) = &client.user {
                if team_members.contains(user_uuid) {
                    client.pending_responses.push(event.clone());
                }
            }
        }
    }

    pub fn broadcast_to_team_except(
        &mut self,
        team_members: &[String],
        exclude_uuid: &str,
        event: Response,
    ) {
        for client in self.clients.values_mut() {
            if let Some(user_uuid) = &client.user {
                if user_uuid != exclude_uuid && team_members.contains(user_uuid) {
                    client.pending_responses.push(event.clone());
                }
            }
        }
    }

    pub fn send_to_user(&mut self, target_uuid: &str, event: Response) {
        for client in self.clients.values_mut() {
            if let Some(user_uuid) = &client.user {
                if user_uuid == target_uuid {
                    client.pending_responses.push(event.clone());
                }
            }
        }
    }

    // server serialization to file

    pub fn save(&mut self) {
        for client in self.clients.values_mut() {
            let _ = client.socket.shutdown(std::net::Shutdown::Both);
        }

        self.clients.clear();

        let mut file = File::create("./state.txt").expect("Error creating state file");
        write_length(&mut file, self.users.len() as u64).expect("Error writing number of users");
        for user in &self.users {
            user.1.write_to_file(&mut file);
        }

        write_length(&mut file, self.private_messages.len() as u64)
            .expect("Failed to write the message length");
        for message in &self.private_messages {
            message.write_to_file(&mut file);
        }

        write_length(&mut file, self.teams.len() as u64).expect("Failed to write messages length");
        for team in &self.teams {
            team.1.write_to_file(&mut file);
        }
    }

    pub fn load(&mut self) {
        let mut file = match File::open("./state.txt") {
            Ok(f) => f,
            Err(_) => return,
        };

        let user_count = read_length(&mut file).unwrap();
        for _ in 0..user_count {
            let user = User::read_from_file(&mut file);
            self.users.insert(user.uuid.clone(), user);
        }

        let msg_count = read_length(&mut file).unwrap();
        for _ in 0..msg_count {
            self.private_messages
                .push(Message::read_from_file(&mut file));
        }

        let teams_count = read_length(&mut file).unwrap();
        for _ in 0..teams_count {
            let team = Team::read_from_file(&mut file);
            self.teams.insert(team.uuid.clone(), team);
        }
    }
}
