use crate::client::Cli;
use myteams::common::StatusCode;
use myteams::common::protocol::ResponseCode;
use myteams::common::protocol::command::Command;
use myteams::common::protocol::request::Request;
use myteams::common::utils::parse_args;
use myteams::{client_error_unauthorized, client_print_users};
use std::io;

pub fn handle_users_request(cli: &mut Cli) -> io::Result<()> {
    cli.pending_request = Some(Request {
        command: Command::Users,
    });
    if cli.handle_request().is_err() {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            "Failed to send users request to server",
        ));
    }
    Ok(())
}

pub fn handle_users_response(cli: &mut Cli) -> io::Result<()> {
    let response = cli.handle_response();
    if response.is_none() {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            "Failed to read response from server",
        ));
    }
    let response = response.unwrap();
    match response.code {
        ResponseCode::Status(StatusCode::Ok) => {
            if let Some(data) = response.data {
                let args = parse_args(data.as_str());
                let mut i = 0;
                while i + 2 < args.len() {
                    let uuid = &args[i];
                    let name = &args[i + 1];
                    let status = args[i + 2].parse::<i32>().unwrap_or(0);
                    client_print_users(uuid, name, status);
                    i += 3;
                }
            }
            return Ok(());
        }
        ResponseCode::Status(StatusCode::Unauthorized) => {
            client_error_unauthorized();
            return Ok(());
        }
        _ => println!("Unexpected response from server: {:?}", response),
    }
    Err(io::Error::new(
        io::ErrorKind::Other,
        "Failed to handle users response",
    ))
}
