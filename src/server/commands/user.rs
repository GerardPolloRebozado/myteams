use crate::server::Server;
use myteams::common::protocol::ResponseCode;
use myteams::common::{Response, StatusCode};

pub fn handle_users(server: &mut Server, client_uuid: &str) {
    let mut users_list = "".to_string();

    for user in server.users.values() {
        let status = if server.clients.values().any(|c| c.user.as_ref() == Some(&user.uuid)) {
            1
        } else {
            0
        };
        users_list.push_str(&format!("\"{}\" \"{}\" \"{}\" ", user.uuid, user.name, status));
    }
    let client = server.clients.get_mut(client_uuid).unwrap();
    client.pending_responses.push(Response::new(
        ResponseCode::Status(StatusCode::Ok),
        Some(users_list),
    ));
}

pub fn handle_user(server: &mut Server, client_uuid: &str, user_uuid: &str) {
    let user = server.users.get(user_uuid);
    if user.is_none() {
        let client = server.clients.get_mut(client_uuid).unwrap();
        client.pending_responses.push(Response::new(
            ResponseCode::Status(StatusCode::NotFound),
            Some(user_uuid.to_string()),
        ));
        return;
    }
    let user = user.unwrap();
    let status = if server.clients.values().any(|c| c.user.as_ref() == Some(&user.uuid)) {
        1
    } else {
        0
    };
    let data = format!("\"{}\" \"{}\" \"{}\"", user.uuid, user.name, status);
    let client = server.clients.get_mut(client_uuid).unwrap();
    client.pending_responses.push(Response::new(
        ResponseCode::Status(StatusCode::Ok),
        Some(data),
    ))
}
