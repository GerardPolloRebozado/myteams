use crate::client::Cli;
use myteams::common::protocol::ResponseCode;
use myteams::common::utils::parse_args;
use myteams::common::{Command, Request, StatusCode};
use myteams::{client_error_unauthorized, client_error_unknown_user, client_print_user};
use std::io;

pub fn handle_user_request(cli: &mut Cli, cmd: &str) -> io::Result<()> {
    let args = parse_args(cmd);
    if args.len() < 2 {
        println!("Usage: /user \"user_uuid\"");
        return Err(io::Error::new(io::ErrorKind::Other, "Invalid usage"));
    }
    let user_uuid = args[1].clone();

    cli.pending_request = Some(Request {
        command: Command::User(user_uuid),
    });
    if cli.handle_request().is_err() {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            "Failed to send users request to server",
        ));
    }
    Ok(())
}

pub fn handle_user_response(cli: &mut Cli) -> io::Result<()> {
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
                let args = parse_args(&data);
                if args.len() >= 3 {
                    let status = args[2].parse::<i32>().unwrap_or(0);
                    client_print_user(&args[0], &args[1], status);
                }
            }
            return Ok(());
        }
        ResponseCode::Status(StatusCode::Unauthorized) => {
            client_error_unauthorized();
            return Ok(());
        }
        ResponseCode::Status(StatusCode::NotFound) => {
            if let Some(uuid) = response.data {
                client_error_unknown_user(&uuid);
            }
            return Ok(());
        }
        _ => println!("Unexpected response from server: {:?}", response),
    }
    Err(io::Error::new(
        io::ErrorKind::Other,
        "Failed to handle user response",
    ))
}
