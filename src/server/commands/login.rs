use crate::server::Server;
use myteams::common::protocol::response::{Response, ResponseCode};
use myteams::common::protocol::status::StatusCode;
use myteams::common::user::User;
use myteams::common::utils::escape_str;
use myteams::common::utils::constants::MAX_NAME_LENGTH;
use myteams::{server_event_user_created_safe, server_event_user_logged_in_safe};

pub fn handle_login(server: &mut Server, client_uuid: &str, name: String) {
    if name.len() > MAX_NAME_LENGTH {
        let response = Response {
            code: ResponseCode::Status(StatusCode::BadRequest),
            data: None,
        };
        if let Some(client) = server.clients.get_mut(client_uuid) {
            client.pending_responses.push(response);
        }
        return;
    }

    let user_uuid = resolve_user(server, &name);

    perform_login(server, client_uuid, user_uuid.clone());

    send_success_response(server, client_uuid, &user_uuid, &name);
}

fn resolve_user(server: &mut Server, name: &String) -> String {
    if let Some(user) = server.users.values().find(|u| &u.name == name) {
        user.uuid.clone()
    } else {
        create_new_user(server, name)
    }
}

fn create_new_user(server: &mut Server, name: &String) -> String {
    let user = User::new(name.clone());
    let uuid = user.uuid.clone();
    server_event_user_created_safe(&uuid, name);
    server.users.insert(uuid.clone(), user);
    // TODO: Persistence save
    uuid.clone()
}

fn perform_login(server: &mut Server, client_uuid: &str, user_uuid: String) {
    if let Some(client) = server.clients.get_mut(client_uuid) {
        client.user = Some(user_uuid.clone());
    }

    server_event_user_logged_in_safe(user_uuid.as_str());
}

fn send_success_response(
    server: &mut Server,
    client_uuid: &str,
    user_uuid: &String,
    name: &String,
) {
    let response = Response {
        code: ResponseCode::Status(StatusCode::Ok),
        data: Some(format!("\"{}\" \"{}\"", user_uuid, escape_str(name))),
    };
    if let Some(client) = server.clients.get_mut(client_uuid) {
        client.pending_responses.push(response);
    }

    let event_resp = Response {
        code: ResponseCode::Event(myteams::common::protocol::event::EventCode::LoggedIn),
        data: Some(format!("\"{}\" \"{}\"", user_uuid, escape_str(name))),
    };
    server.broadcast_global(event_resp);
}
