use crate::client::Cli;
use myteams::common::protocol::command::Command;
use myteams::common::protocol::request::Request;
use myteams::common::utils::parse_args;
use std::io;

pub fn handle_use_request(cli: &mut Cli, cmd: &str) -> io::Result<()> {
    let args = parse_args(cmd);
    
    let team_uuid = args.get(1).map(|s| s.to_string());
    let channel_uuid = args.get(2).map(|s| s.to_string());
    let thread_uuid = args.get(3).map(|s| s.to_string());

    cli.pending_request = Some(Request {
        command: Command::Use(team_uuid, channel_uuid, thread_uuid),
    });

    if cli.handle_request().is_err() {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            "Failed to send use request to server",
        ));
    }

    Ok(())
}

pub fn handle_use_response(cli: &mut Cli) -> io::Result<()> {
    let response = cli.handle_response();
    if response.is_none() {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            "Failed to read response from server",
        ));
    }
    Ok(())
}
