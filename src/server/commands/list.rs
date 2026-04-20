use crate::server::Server;
use myteams::common::protocol::response::{Response, ResponseCode};
use myteams::common::protocol::status::StatusCode;

pub fn handle_list(server: &mut Server, client_uuid: &str) {
    let current_user_uuid = match get_authenticated_user(server, client_uuid) {
        Some(uuid) => uuid,
        None => {
            set_response(server, client_uuid, StatusCode::Unauthorized, None);
            return;
        }
    };

    let (team_ctx, channel_ctx, thread_ctx) = get_client_context(server, client_uuid);

    let (status, data) = match (team_ctx, channel_ctx, thread_ctx) {
        (None, None, None) => list_teams(server),
        (Some(t), None, None) => list_channels(server, &t, &current_user_uuid),
        (Some(t), Some(c), None) => list_threads(server, &t, &c, &current_user_uuid),
        (Some(t), Some(c), Some(th)) => list_replies(server, &t, &c, &th, &current_user_uuid),
        _ => (StatusCode::BadRequest, None),
    };

    set_response(server, client_uuid, status, data);
}

fn get_authenticated_user(server: &Server, client_uuid: &str) -> Option<String> {
    server.clients.get(client_uuid).and_then(|c| c.user.clone())
}

fn get_client_context(
    server: &Server,
    client_uuid: &str,
) -> (Option<String>, Option<String>, Option<String>) {
    let client = server.clients.get(client_uuid).unwrap();
    (
        client.client_ctx.team_uuid.clone(),
        client.client_ctx.channel_uuid.clone(),
        client.client_ctx.thread_uuid.clone(),
    )
}

fn set_response(server: &mut Server, client_uuid: &str, status: StatusCode, data: Option<String>) {
    let resp = Response {
        code: ResponseCode::Status(status),
        data,
    };

    if let Some(client) = server.clients.get_mut(client_uuid) {
        client.pending_responses.push(resp);
    }
}

fn list_teams(server: &Server) -> (StatusCode, Option<String>) {
    let mut data = String::from("TEAM");

    for team in server.teams.values() {
        data.push(' ');
        data.push_str(&format!(
            "\"{}\" \"{}\" \"{}\"",
            team.uuid, team.name, team.description
        ));
    }

    (StatusCode::Ok, Some(data))
}

fn list_channels(
    server: &Server,
    team_uuid: &str,
    _user_uuid: &str,
) -> (StatusCode, Option<String>) {
    let team = match server.teams.get(team_uuid) {
        Some(t) => t,
        None => {
            return (
                StatusCode::NotFound,
                Some(format!("TEAM \"{}\"", team_uuid)),
            );
        }
    };

    let mut data = String::from("CHANNEL");

    for channel in team.channels.values() {
        data.push(' ');
        data.push_str(&format!(
            "\"{}\" \"{}\" \"{}\"",
            channel.uuid, channel.name, channel.description
        ));
    }

    (StatusCode::Ok, Some(data))
}

fn list_threads(
    server: &Server,
    team_uuid: &str,
    channel_uuid: &str,
    _user_uuid: &str,
) -> (StatusCode, Option<String>) {
    if !server.teams.contains_key(team_uuid) {
        return (
            StatusCode::NotFound,
            Some(format!("TEAM \"{}\"", team_uuid)),
        );
    }
    let team = server.teams.get(team_uuid).unwrap();

    let channel = match team.channels.get(channel_uuid) {
        Some(c) => c,
        None => {
            return (
                StatusCode::NotFound,
                Some(format!("CHANNEL \"{}\"", channel_uuid)),
            );
        }
    };

    let mut data = String::from("THREAD");

    for thread in channel.threads.values() {
        data.push(' ');

        let (user_uuid, body) = thread
            .messages
            .first()
            .map(|m| (m.sender_uuid.clone(), m.message.clone()))
            .unwrap_or((String::new(), String::new()));

        data.push_str(&format!(
            "\"{}\" \"{}\" \"{}\" \"{}\" \"{}\"",
            thread.uuid,
            user_uuid,
            thread.creation_date.to_timestamp(),
            thread.title,
            body
        ));
    }

    (StatusCode::Ok, Some(data))
}

fn list_replies(
    server: &Server,
    team_uuid: &str,
    channel_uuid: &str,
    thread_uuid: &str,
    _user_uuid: &str,
) -> (StatusCode, Option<String>) {
    if !server.teams.contains_key(team_uuid) {
        return (
            StatusCode::NotFound,
            Some(format!("TEAM \"{}\"", team_uuid)),
        );
    }
    let team = server.teams.get(team_uuid).unwrap();

    if !team.channels.contains_key(channel_uuid) {
        return (
            StatusCode::NotFound,
            Some(format!("CHANNEL \"{}\"", channel_uuid)),
        );
    }
    let channel = team.channels.get(channel_uuid).unwrap();

    let thread = match channel.threads.get(thread_uuid) {
        Some(t) => t,
        None => {
            return (
                StatusCode::NotFound,
                Some(format!("THREAD \"{}\"", thread_uuid)),
            );
        }
    };

    let mut data = String::from("REPLY");

    for msg in thread.messages.iter() {
        data.push(' ');

        data.push_str(&format!(
            "\"{}\" \"{}\" \"{}\" \"{}\"",
            thread.uuid,
            msg.sender_uuid,
            msg.date.to_timestamp(),
            msg.message
        ));
    }

    (StatusCode::Ok, Some(data))
}
