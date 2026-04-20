use crate::server::Server;
use myteams::common::message::{Location, Message};
use myteams::common::protocol::response::{Response, ResponseCode};
use myteams::common::protocol::status::StatusCode;
use myteams::common::utils::escape_str;
use myteams::common::utils::constants::MAX_BODY_LENGTH;
use myteams::server_event_private_message_sended;

pub fn handle_send(server: &mut Server, client_uuid: &str, receiver_uuid: String, body: String) {
    if body.len() > MAX_BODY_LENGTH {
        let resp = Response {
            code: ResponseCode::Status(StatusCode::BadRequest),
            data: None,
        };
        if let Some(client) = server.clients.get_mut(client_uuid) {
            client.pending_responses.push(resp);
        }
        return;
    }
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

    if !server.users.contains_key(&receiver_uuid) {
        let resp = Response {
            code: ResponseCode::Status(StatusCode::NotFound),
            data: Some(receiver_uuid),
        };
        if let Some(client) = server.clients.get_mut(client_uuid) {
            client.pending_responses.push(resp);
        }
        return;
    }

    let sender_user = server.users.get(&current_user_uuid).unwrap();
    let msg = Message::new(
        sender_user.uuid.clone(),
        Location::USER,
        receiver_uuid.clone(),
        body.clone(),
    );

    server.private_messages.push(msg);

    server_event_private_message_sended(&current_user_uuid, &receiver_uuid, &body);

    let resp = Response {
        code: ResponseCode::Status(StatusCode::Ok),
        data: None,
    };
    if let Some(client) = server.clients.get_mut(client_uuid) {
        client.pending_responses.push(resp);
    }

    let sender_uuid = current_user_uuid.clone();
    let event_resp = Response {
        code: ResponseCode::Event(myteams::common::protocol::event::EventCode::MessageReceived),
        data: Some(format!("\"{}\" \"{}\"", sender_uuid, escape_str(&body))),
    };

    server.send_to_user(&receiver_uuid, event_resp);
}
