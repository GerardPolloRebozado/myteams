use crate::server::Server;
use myteams::common::protocol::ResponseCode::Status;
use myteams::common::{Response, StatusCode};

pub fn handle_subscribed_users(server: &mut Server, client_uuid: &str, team_uuid: String) {
    let team = server.teams.get(&team_uuid);
    if team.is_none() {
        let client = server.clients.get_mut(client_uuid).unwrap();
        client.pending_responses.push(
            Response::new(Status(StatusCode::NotFound), Some(team_uuid)));
        return;
    }
    let team = team.unwrap();
    let mut users_list = String::new();

    for user_uuid in &team.users_uuid {
        let user = server.users.get(user_uuid);
        if let Some(user) = user {
            let status = if server.clients.values().any(|c| c.user.as_ref() == Some(&user.uuid)) {
                1
            } else {
                0
            };
            users_list.push_str(&format!("\"{}\" \"{}\" \"{}\" ", user.uuid, user.name, status));
        }
    }
    let client = server.clients.get_mut(client_uuid).unwrap();
    client.pending_responses.push(Response::new(Status(StatusCode::Ok), Some(users_list)));
}

pub fn handle_subscribed_teams(server: &mut Server, client_uuid: &str) {
    let mut subscribed_teams = String::new();
    let client = server.clients.get(client_uuid).unwrap();
    let user_uuid = client.user.clone().unwrap();

    for team in server.teams.values() {
        if team.users_uuid.contains(&user_uuid) {
            subscribed_teams.push_str(&format!(
                "\"{}\" \"{}\" \"{}\" ",
                team.uuid, team.name, team.description
            ));
        }
    }
    let client = server.clients.get_mut(client_uuid).unwrap();
    client.pending_responses.push(Response::new(
        Status(StatusCode::Ok),
        Some(subscribed_teams),
    ));
}
