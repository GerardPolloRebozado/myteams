use crate::server::Server;
use myteams::common::protocol::ResponseCode::Status;
use myteams::common::{Response, StatusCode};
use myteams::server_event_user_subscribed;

pub fn handle_subscribe(server: &mut Server, client_uuid: &str, team_uuid: &str) {
    let client = server.clients.get_mut(client_uuid).unwrap();

    let team_to_subscribe = server.teams.get_mut(team_uuid);
    if team_to_subscribe.is_none() {
        client.pending_responses.push(Response::new(
            Status(StatusCode::NotFound),
            Some(team_uuid.to_string()),
        ));
        return;
    }

    let user_uuid = client.user.clone().unwrap();
    let team = team_to_subscribe.unwrap();
    if team.users_uuid.contains(&user_uuid) {
        client.pending_responses.push(Response::new(
            Status(StatusCode::Conflict),
            Some(client_uuid.to_string()),
        ));
        return;
    }
    team.users_uuid.push(user_uuid.to_string());
    client.pending_responses.push(Response::new(
        Status(StatusCode::Ok),
        Some(format!("\"{}\" \"{}\"", user_uuid, team.uuid)),
    ));
    server_event_user_subscribed(user_uuid.as_str(), team.uuid.as_str());
}
