use crate::server::Server;
use myteams::common::protocol::ResponseCode;
use myteams::common::Response;
use myteams::common::StatusCode;

pub fn handle_use(
    server: &mut Server,
    client_uuid: &str,
    team_uuid: Option<String>,
    channel_uuid: Option<String>,
    thread_uuid: Option<String>,
) {
    let client = match server.clients.get_mut(client_uuid) {
        Some(c) => c,
        None => return,
    };

    client.client_ctx.team_uuid = team_uuid;
    client.client_ctx.channel_uuid = channel_uuid;
    client.client_ctx.thread_uuid = thread_uuid;

    client.pending_responses.push(Response {
        code: ResponseCode::Status(StatusCode::Ok),
        data: None,
    });
}
