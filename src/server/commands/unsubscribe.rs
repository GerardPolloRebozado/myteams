use crate::server::Server;
use myteams::common::protocol::ResponseCode;
use myteams::common::{Response, StatusCode};
use myteams::server_event_user_unsubscribed;

pub fn handle_unsubscribe(server: &mut Server, client_uuid: &str, team_uuid: &str) {
    let client = server.clients.get_mut(client_uuid);
    if client.is_none() {
        return;
    }
    let client = client.unwrap();
    let team = server.teams.get_mut(team_uuid);
    if team.is_none() {
        client.pending_responses.push(Response::new(
            ResponseCode::Status(StatusCode::NotFound),
            Some(format!("Team: {} not found", team_uuid)),
        ));
        return;
    }
    let team = team.unwrap();
    let user_uuid = client.user.clone().unwrap();
    let user_index = team.users_uuid.iter().position(|u| u == &user_uuid);
    if user_index.is_none() {
        client.pending_responses.push(Response::new(
            ResponseCode::Status(StatusCode::NotFound),
            Some(format!(
                "User: {} in team: {} not found",
                user_uuid, team_uuid
            )),
        ));
        return;
    }
    team.users_uuid.remove(user_index.unwrap());
    client.pending_responses.push(Response::new(
        ResponseCode::Status(StatusCode::Ok),
        Some(format!("\"{}\" \"{}\"", user_uuid, team.uuid)),
    ));
    server_event_user_unsubscribed(user_uuid.as_str(), team.uuid.as_str());
}
