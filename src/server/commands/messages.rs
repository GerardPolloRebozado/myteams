use crate::server::Server;
use myteams::common::message::Location;
use myteams::common::protocol::response::{Response, ResponseCode};
use myteams::common::protocol::status::StatusCode;

pub fn handle_messages(server: &mut Server, client_uuid: &str, other_user_uuid: String) {
    let current_user_uuid = match server.clients.get(client_uuid).and_then(|c| c.user.clone()) {
        Some(uuid) => uuid,
        None => {
            let resp = Response {
                code: ResponseCode::Status(StatusCode::Unauthorized),
                data: None,
            };
            if let Some(client) = server.clients.get_mut(client_uuid) {
                client.pending_responses.push(resp);
            }
            return;
        }
    };

    if !server.users.contains_key(&other_user_uuid) {
        let resp = Response {
            code: ResponseCode::Status(StatusCode::NotFound),
            data: Some(other_user_uuid),
        };
        if let Some(client) = server.clients.get_mut(client_uuid) {
            client.pending_responses.push(resp);
        }
        return;
    }
    let filtered_messages: Vec<_> = server
        .private_messages
        .iter()
        .filter(|m| {
            if let Location::USER = m.location_type {
                (m.sender_uuid == current_user_uuid && m.location_uuid == other_user_uuid)
                    || (m.sender_uuid == other_user_uuid && m.location_uuid == current_user_uuid)
            } else {
                false
            }
        })
        .collect();

    let mut data = String::new();
    for (i, m) in filtered_messages.iter().enumerate() {
        if i > 0 {
            data.push(' ');
        }
        let timestamp = m.date.to_timestamp();
        data.push_str(&format!(
            "\"{}\" \"{}\" \"{}\"",
            m.sender_uuid, timestamp, m.message
        ));
    }

    let resp = Response {
        code: ResponseCode::Status(StatusCode::Ok),
        data: if data.is_empty() { None } else { Some(data) },
    };
    if let Some(client) = server.clients.get_mut(client_uuid) {
        client.pending_responses.push(resp);
    }
}
