use crate::server::Server;
use myteams::common::protocol::response::{Response, ResponseCode};
use myteams::common::protocol::status::StatusCode;

pub fn handle_info(server: &mut Server, client_uuid: &str) {
    let current_user_uuid = match get_authenticated_user(server, client_uuid) {
        Some(uuid) => uuid,
        None => {
            set_response(server, client_uuid, StatusCode::Unauthorized, None);
            return;
        }
    };

    let (team_ctx, channel_ctx, thread_ctx) = get_client_context(server, client_uuid);

    let (status, data) = match (team_ctx, channel_ctx, thread_ctx) {
        (None, None, None) => get_user_info(server, &current_user_uuid),
        (Some(t), None, None) => get_team_info(server, &t),
        (Some(t), Some(c), None) => get_channel_info(server, &t, &c),
        (Some(t), Some(c), Some(th)) => get_thread_info(server, &t, &c, &th),
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

fn get_user_info(server: &Server, user_uuid: &str) -> (StatusCode, Option<String>) {
    let user = match server.users.get(user_uuid) {
        Some(u) => u,
        None => {
            return (
                StatusCode::NotFound,
                Some(format!("USER \"{}\"", user_uuid)),
            );
        }
    };

    let status = if server
        .clients
        .values()
        .any(|c| c.user.as_ref() == Some(&user.uuid))
    {
        1
    } else {
        0
    };

    (
        StatusCode::Ok,
        Some(format!(
            "USER \"{}\" \"{}\" \"{}\"",
            user.uuid, user.name, status
        )),
    )
}

fn get_team_info(server: &Server, team_uuid: &str) -> (StatusCode, Option<String>) {
    let team = match server.teams.get(team_uuid) {
        Some(t) => t,
        None => {
            return (
                StatusCode::NotFound,
                Some(format!("TEAM \"{}\"", team_uuid)),
            );
        }
    };

    (
        StatusCode::Ok,
        Some(format!(
            "TEAM \"{}\" \"{}\" \"{}\"",
            team.uuid, team.name, team.description
        )),
    )
}

fn get_channel_info(
    server: &Server,
    team_uuid: &str,
    channel_uuid: &str,
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

    let channel = match team.channels.get(channel_uuid) {
        Some(c) => c,
        None => {
            return (
                StatusCode::NotFound,
                Some(format!("CHANNEL \"{}\"", channel_uuid)),
            );
        }
    };

    (
        StatusCode::Ok,
        Some(format!(
            "CHANNEL \"{}\" \"{}\" \"{}\"",
            channel.uuid, channel.name, channel.description
        )),
    )
}

fn get_thread_info(
    server: &Server,
    team_uuid: &str,
    channel_uuid: &str,
    thread_uuid: &str,
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

    let channel = match team.channels.get(channel_uuid) {
        Some(c) => c,
        None => {
            return (
                StatusCode::NotFound,
                Some(format!("CHANNEL \"{}\"", channel_uuid)),
            );
        }
    };

    let thread = match channel.threads.get(thread_uuid) {
        Some(t) => t,
        None => {
            return (
                StatusCode::NotFound,
                Some(format!("THREAD \"{}\"", thread_uuid)),
            );
        }
    };

    let (user_uuid, body) = thread
        .messages
        .first()
        .map(|m| (m.sender_uuid.clone(), m.message.clone()))
        .unwrap_or((String::new(), String::new()));

    (
        StatusCode::Ok,
        Some(format!(
            "THREAD \"{}\" \"{}\" \"{}\" \"{}\" \"{}\"",
            thread.uuid,
            user_uuid,
            thread.creation_date.to_timestamp(),
            thread.title,
            body
        )),
    )
}
