use crate::server::Server;
use myteams::common::message::Location;
use myteams::common::protocol::ResponseCode::Status;
use myteams::common::utils::constants::{
    MAX_BODY_LENGTH, MAX_DESCRIPTION_LENGTH, MAX_NAME_LENGTH,
};
use myteams::common::{Channel, Message, Response, StatusCode, Team, Thread};
use myteams::{
    server_event_channel_created, server_event_reply_created, server_event_team_created,
    server_event_thread_created,
};
use myteams::common::protocol::ResponseCode;
use myteams::common::utils::escape_str;

fn handle_create_team(
    server: &mut Server,
    client_uuid: &str,
    team_name: String,
    team_description: String,
) {
    if team_name.len() > MAX_NAME_LENGTH || team_description.len() > MAX_DESCRIPTION_LENGTH {
        let client = server.clients.get_mut(client_uuid).unwrap();
        client.pending_responses.push(Response::new(Status(StatusCode::BadRequest), None));
        return;
    }
    let user_uuid = match server.clients.get(client_uuid).and_then(|c| c.user.clone()) {
        Some(uuid) => uuid,
        None => {
            let client = server.clients.get_mut(client_uuid).unwrap();
            client.pending_responses.push(Response::new(Status(StatusCode::Unauthorized), None));
            return;
        }
    };

    let existing_team = server.teams.values().find(|t| t.name == team_name);
    if existing_team.is_some() {
        let client = server.clients.get_mut(client_uuid).unwrap();
        client.pending_responses.push(Response::new(
            Status(StatusCode::Conflict),
            Some(format!(
                "\"{}\" \"{}\" \"{}\"",
                existing_team.unwrap().uuid,
                escape_str(&team_name),
                escape_str(&existing_team.unwrap().description)
            )),
        ));
        return;
    }

    let mut new_team = Team::new(team_name);
    new_team.users_uuid.push(user_uuid.clone());
    new_team.description = team_description;

    server_event_team_created(&new_team.uuid, &new_team.name, &user_uuid);

    let response_data = format!(
        "TEAM \"{}\" \"{}\" \"{}\"",
        new_team.uuid,
        escape_str(&new_team.name),
        escape_str(&new_team.description)
    );

    let event_data = format!(
        "\"{}\" \"{}\" \"{}\"",
        new_team.uuid,
        escape_str(&new_team.name),
        escape_str(&new_team.description)
    );

    server.teams.insert(new_team.uuid.clone(), new_team);

    if let Some(client) = server.clients.get_mut(client_uuid) {
        client.pending_responses.push(Response::new(
            Status(StatusCode::Ok),
            Some(response_data),
        ));
    }

    let event_resp = Response {
        code: ResponseCode::Event(myteams::common::protocol::event::EventCode::TeamCreated),
        data: Some(event_data),
    };
    server.broadcast_global(event_resp);
}

fn handle_create_channel(
    server: &mut Server,
    client_uuid: &str,
    team_uuid: &str,
    channel_name: String,
    channel_description: String,
) {
    if channel_name.len() > MAX_NAME_LENGTH || channel_description.len() > MAX_DESCRIPTION_LENGTH {
        let client = server.clients.get_mut(client_uuid).unwrap();
        client.pending_responses.push(Response::new(Status(StatusCode::BadRequest), None));
        return;
    }
    let user_uuid = match server.clients.get(client_uuid).and_then(|c| c.user.clone()) {
        Some(uuid) => uuid,
        None => {
            let client = server.clients.get_mut(client_uuid).unwrap();
            client.pending_responses.push(Response::new(Status(StatusCode::Unauthorized), None));
            return;
        }
    };

    let team = match server.teams.get_mut(team_uuid) {
        Some(t) => t,
        None => {
            let client = server.clients.get_mut(client_uuid).unwrap();
            client.pending_responses.push(Response::new(
                Status(StatusCode::NotFound),
                Some(format!("TEAM \"{}\"", team_uuid)),
            ));
            return;
        }
    };

    if !team.users_uuid.contains(&user_uuid) {
        let client = server.clients.get_mut(client_uuid).unwrap();
        client.pending_responses.push(Response::new(Status(StatusCode::Unauthorized), None));
        return;
    }

    let existing_channel = team.channels.values().find(|c| c.name == channel_name);
    if existing_channel.is_some() {
        let client = server.clients.get_mut(client_uuid).unwrap();
        client.pending_responses.push(Response::new(
            Status(StatusCode::Conflict),
            Some(format!(
                "\"{}\" \"{}\" \"{}\"",
                existing_channel.unwrap().uuid,
                escape_str(&channel_name),
                escape_str(&existing_channel.unwrap().description)
            )),
        ));
        return;
    }

    let mut new_channel = Channel::new(&channel_name);
    new_channel.description = channel_description;

    server_event_channel_created(team_uuid, &new_channel.uuid, &new_channel.name);

    let response_data = format!(
        "CHANNEL \"{}\" \"{}\" \"{}\"",
        new_channel.uuid,
        escape_str(&new_channel.name),
        escape_str(&new_channel.description)
    );

    let event_data = format!(
        "\"{}\" \"{}\" \"{}\"",
        new_channel.uuid,
        escape_str(&new_channel.name),
        escape_str(&new_channel.description)
    );

    let team_members = team.users_uuid.clone();
    team.channels.insert(new_channel.uuid.clone(), new_channel);

    if let Some(client) = server.clients.get_mut(client_uuid) {
        client.pending_responses.push(Response::new(
            Status(StatusCode::Ok),
            Some(response_data),
        ));
    }

    let event_resp = Response {
        code: ResponseCode::Event(myteams::common::protocol::event::EventCode::ChannelCreated),
        data: Some(event_data),
    };
    server.broadcast_to_team_except(&team_members, &user_uuid, event_resp);
}

fn handle_create_thread(
    server: &mut Server,
    client_uuid: &str,
    team_uuid: &str,
    channel_uuid: &str,
    thread_title: String,
    thread_message: String,
) {
    if thread_title.len() > MAX_NAME_LENGTH || thread_message.len() > MAX_BODY_LENGTH {
        let client = server.clients.get_mut(client_uuid).unwrap();
        client.pending_responses.push(Response::new(Status(StatusCode::BadRequest), None));
        return;
    }
    let user_uuid = match server.clients.get(client_uuid).and_then(|c| c.user.clone()) {
        Some(uuid) => uuid,
        None => {
            let client = server.clients.get_mut(client_uuid).unwrap();
            client.pending_responses.push(Response::new(Status(StatusCode::Unauthorized), None));
            return;
        }
    };

    let team = match server.teams.get_mut(team_uuid) {
        Some(t) => t,
        None => {
            let client = server.clients.get_mut(client_uuid).unwrap();
            client.pending_responses.push(Response::new(
                Status(StatusCode::NotFound),
                Some(format!("TEAM \"{}\"", team_uuid)),
            ));
            return;
        }
    };

    if !team.users_uuid.contains(&user_uuid) {
        let client = server.clients.get_mut(client_uuid).unwrap();
        client.pending_responses.push(Response::new(Status(StatusCode::Unauthorized), None));
        return;
    }

    let channel = match team.channels.get_mut(channel_uuid) {
        Some(c) => c,
        None => {
            let client = server.clients.get_mut(client_uuid).unwrap();
            client.pending_responses.push(Response::new(
                Status(StatusCode::NotFound),
                Some(format!("CHANNEL \"{}\"", channel_uuid)),
            ));
            return;
        }
    };

    let existing_thread = channel.threads.values().find(|t| t.title == thread_title);
    if existing_thread.is_some() {
        let client = server.clients.get_mut(client_uuid).unwrap();
        client.pending_responses.push(Response::new(
            Status(StatusCode::Conflict),
            Some(format!(
                "\"{}\" \"{}\" \"{}\" \"{}\" \"{}\"",
                existing_thread.unwrap().uuid,
                user_uuid,
                existing_thread.unwrap().creation_date.to_timestamp(),
                escape_str(&thread_title),
                escape_str(
                    existing_thread
                        .unwrap()
                        .messages
                        .first()
                        .map_or("", |m| m.message.as_str())
                )
            )),
        ));
        return;
    }

    let mut new_thread = Thread::new(thread_title.clone());
    let message = Message::new(
        user_uuid.clone(),
        Location::THREAD,
        new_thread.uuid.clone(),
        thread_message.clone(),
    );
    new_thread.messages.push(message.clone());

    server_event_thread_created(
        channel_uuid,
        &new_thread.uuid,
        &user_uuid,
        &new_thread.title,
        &message.message,
    );

    let response_data = format!(
        "THREAD \"{}\" \"{}\" \"{}\" \"{}\" \"{}\"",
        new_thread.uuid,
        user_uuid,
        new_thread.creation_date.to_timestamp(),
        escape_str(&new_thread.title),
        escape_str(&message.message)
    );

    let event_data = format!(
        "\"{}\" \"{}\" \"{}\" \"{}\" \"{}\"",
        new_thread.uuid,
        user_uuid,
        new_thread.creation_date.to_timestamp(),
        escape_str(&new_thread.title),
        escape_str(&message.message)
    );

    let team_members = team.users_uuid.clone();
    channel
        .threads
        .insert(new_thread.uuid.clone(), new_thread);

    if let Some(client) = server.clients.get_mut(client_uuid) {
        client.pending_responses.push(Response::new(
            Status(StatusCode::Ok),
            Some(response_data),
        ));
    }

    let event_resp = Response {
        code: ResponseCode::Event(myteams::common::protocol::event::EventCode::ThreadCreated),
        data: Some(event_data),
    };
    server.broadcast_to_team_except(&team_members, &user_uuid, event_resp);
}

fn handle_create_comment(
    server: &mut Server,
    client_uuid: &str,
    team_uuid: &str,
    channel_uuid: &str,
    thread_uuid: &str,
    comment_body: String,
) {
    if comment_body.len() > MAX_BODY_LENGTH {
        let client = server.clients.get_mut(client_uuid).unwrap();
        client.pending_responses.push(Response::new(Status(StatusCode::BadRequest), None));
        return;
    }
    let user_uuid = match server.clients.get(client_uuid).and_then(|c| c.user.clone()) {
        Some(uuid) => uuid,
        None => {
            let client = server.clients.get_mut(client_uuid).unwrap();
            client.pending_responses.push(Response::new(Status(StatusCode::Unauthorized), None));
            return;
        }
    };

    let team = match server.teams.get_mut(team_uuid) {
        Some(t) => t,
        None => {
            let client = server.clients.get_mut(client_uuid).unwrap();
            client.pending_responses.push(Response::new(
                Status(StatusCode::NotFound),
                Some(format!("TEAM \"{}\"", team_uuid)),
            ));
            return;
        }
    };

    if !team.users_uuid.contains(&user_uuid) {
        let client = server.clients.get_mut(client_uuid).unwrap();
        client.pending_responses.push(Response::new(Status(StatusCode::Unauthorized), None));
        return;
    }

    let channel = match team.channels.get_mut(channel_uuid) {
        Some(c) => c,
        None => {
            let client = server.clients.get_mut(client_uuid).unwrap();
            client.pending_responses.push(Response::new(
                Status(StatusCode::NotFound),
                Some(format!("CHANNEL \"{}\"", channel_uuid)),
            ));
            return;
        }
    };

    let thread = match channel.threads.get_mut(thread_uuid) {
        Some(t) => t,
        None => {
            let client = server.clients.get_mut(client_uuid).unwrap();
            client.pending_responses.push(Response::new(
                Status(StatusCode::NotFound),
                Some(format!("THREAD \"{}\"", thread_uuid)),
            ));
            return;
        }
    };

    let message = Message::new(
        user_uuid.clone(),
        Location::THREAD,
        thread.uuid.clone(),
        comment_body,
    );

    server_event_reply_created(thread_uuid, &user_uuid, &message.message);

    let response_data = format!(
        "REPLY \"{}\" \"{}\" \"{}\" \"{}\"",
        thread.uuid,
        user_uuid,
        message.date.to_timestamp(),
        escape_str(&message.message)
    );

    let event_data = format!(
        "\"{}\" \"{}\" \"{}\" \"{}\"",
        team_uuid,
        thread.uuid,
        user_uuid,
        escape_str(&message.message)
    );

    let team_members = team.users_uuid.clone();
    thread.messages.push(message);

    if let Some(client) = server.clients.get_mut(client_uuid) {
        client.pending_responses.push(Response::new(
            Status(StatusCode::Ok),
            Some(response_data),
        ));
    }

    let event_resp = Response {
        code: ResponseCode::Event(myteams::common::protocol::event::EventCode::CommentCreated),
        data: Some(event_data),
    };
    server.broadcast_to_team_except(&team_members, &user_uuid, event_resp);
}

pub fn handle_create(server: &mut Server, client_uuid: &str, data: Vec<String>) {
    let (team_ctx, channel_ctx, thread_ctx) = {
        let client = server.clients.get(client_uuid).unwrap();
        (
            client.client_ctx.team_uuid.clone(),
            client.client_ctx.channel_uuid.clone(),
            client.client_ctx.thread_uuid.clone(),
        )
    };

    match (team_ctx, channel_ctx, thread_ctx) {
        (Some(t), Some(c), Some(th)) => {
            if data.len() != 1 {
                let client = server.clients.get_mut(client_uuid).unwrap();
                client.pending_responses.push(Response::new(Status(StatusCode::BadRequest), None));
                return;
            }
            handle_create_comment(server, client_uuid, &t, &c, &th, data[0].clone());
        }
        (Some(t), Some(c), None) => {
            if data.len() != 2 {
                let client = server.clients.get_mut(client_uuid).unwrap();
                client.pending_responses.push(Response::new(Status(StatusCode::BadRequest), None));
                return;
            }
            handle_create_thread(
                server,
                client_uuid,
                &t,
                &c,
                data[0].clone(),
                data[1].clone(),
            );
        }
        (Some(t), None, None) => {
            if data.len() != 2 {
                let client = server.clients.get_mut(client_uuid).unwrap();
                client.pending_responses.push(Response::new(Status(StatusCode::BadRequest), None));
                return;
            }
            handle_create_channel(server, client_uuid, &t, data[0].clone(), data[1].clone());
        }
        (None, None, None) => {
            if data.len() != 2 {
                let client = server.clients.get_mut(client_uuid).unwrap();
                client.pending_responses.push(Response::new(Status(StatusCode::BadRequest), None));
                return;
            }
            handle_create_team(server, client_uuid, data[0].clone(), data[1].clone());
        }
        _ => {
            let client = server.clients.get_mut(client_uuid).unwrap();
            client.pending_responses.push(Response::new(Status(StatusCode::BadRequest), None));
        }
    }
}
