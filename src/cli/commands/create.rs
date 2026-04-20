use crate::client::Cli;
use myteams::common::StatusCode;
use myteams::common::protocol::ResponseCode;
use myteams::common::protocol::command::Command;
use myteams::common::protocol::request::Request;
use myteams::common::utils::constants::{MAX_BODY_LENGTH, MAX_NAME_LENGTH};
use myteams::common::utils::parse_args;
use myteams::{
    client_error_already_exist, client_error_unauthorized, client_error_unknown_channel,
    client_error_unknown_team, client_error_unknown_thread, client_print_channel_created,
    client_print_reply_created, client_print_team_created, client_print_thread_created,
};
use std::io;

pub fn handle_create_request(cli: &mut Cli, cmd: &str) -> io::Result<()> {
    let args = parse_args(cmd);
    let mut create_args = Vec::new();

    for i in 1..args.len() {
        create_args.push(args[i].clone());
    }

    if create_args.len() == 1 {
        if create_args[0].len() > MAX_BODY_LENGTH {
            println!("Error: Comment body too long (max {})", MAX_BODY_LENGTH);
            return Err(io::Error::new(io::ErrorKind::Other, "Comment too long"));
        }
    } else if create_args.len() == 2 {
        if create_args[0].len() > MAX_NAME_LENGTH {
            println!("Error: Name too long (max {})", MAX_NAME_LENGTH);
            return Err(io::Error::new(io::ErrorKind::Other, "Name too long"));
        }
        if create_args[1].len() > MAX_BODY_LENGTH {
            println!(
                "Error: Description or Message too long (max {})",
                MAX_BODY_LENGTH
            );
            return Err(io::Error::new(
                io::ErrorKind::Other,
                "Description/Message too long",
            ));
        }
    }

    cli.pending_request = Some(Request {
        command: Command::Create(create_args),
    });

    cli.handle_request()
}

pub fn handle_create_response(cli: &mut Cli) -> io::Result<()> {
    let response = match cli.handle_response() {
        Some(r) => r,
        None => {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                "Failed to read response from server",
            ));
        }
    };

    match response.code {
        ResponseCode::Status(StatusCode::Ok) => {
            if let Some(data) = response.data {
                let parts = parse_args(&data);
                if parts.is_empty() {
                    return Ok(());
                }

                let tag = &parts[0];
                let info = &parts[1..];

                match tag.as_str() {
                    "TEAM" => {
                        if info.len() >= 3 {
                            client_print_team_created(&info[0], &info[1], &info[2]);
                        }
                    }
                    "CHANNEL" => {
                        if info.len() >= 3 {
                            client_print_channel_created(&info[0], &info[1], &info[2]);
                        }
                    }
                    "THREAD" => {
                        if info.len() >= 5 {
                            client_print_thread_created(
                                &info[0],
                                &info[1],
                                info[2].parse().unwrap_or(0),
                                &info[3],
                                &info[4],
                            );
                        }
                    }
                    "REPLY" => {
                        if info.len() >= 4 {
                            client_print_reply_created(
                                &info[0],
                                &info[1],
                                info[2].parse().unwrap_or(0),
                                &info[3],
                            );
                        }
                    }
                    _ => println!("Unknown resource created: {}", tag),
                }
            }
            Ok(())
        }
        ResponseCode::Status(StatusCode::Unauthorized) => {
            client_error_unauthorized();
            Ok(())
        }
        ResponseCode::Status(StatusCode::NotFound) => {
            if let Some(data) = response.data {
                let parts = parse_args(&data);
                if parts.len() >= 2 {
                    let tag = &parts[0];
                    let uuid = &parts[1];
                    match tag.as_str() {
                        "TEAM" => client_error_unknown_team(uuid),
                        "CHANNEL" => client_error_unknown_channel(uuid),
                        "THREAD" => client_error_unknown_thread(uuid),
                        _ => {
                            println!("Resource not found: {} {}", tag, uuid);
                            0
                        }
                    };
                }
            }
            Ok(())
        }
        ResponseCode::Status(StatusCode::Conflict) => {
            client_error_already_exist();
            Ok(())
        }
        _ => {
            println!("Failed to create resource: {:?}", response.code);
            Ok(())
        }
    }
}
