# G-NWP-400-BAR-4-1-myteams-2

An Epitech second-year project: a client/server messaging platform inspired by
Microsoft Teams, written in Rust. It ships two binaries — `myteams_server` and
`myteams_cli` — that talk to each other over TCP using a small text-based
application protocol.

## Features

- User login/logout with online/offline status
- Teams, channels, threads and comments (hierarchical data model)
- Subscribe / unsubscribe to teams
- Personal (direct) messages between users
- `/use` context navigation (team → channel → thread) with `/create`, `/list`, `/info`
- Async server-pushed events to subscribed clients
- Persistent storage: server restores its state from save files on startup
- Shared FFI library (`libmyteams`) used by the binaries

## Project layout

    .
    ├── src/
    │   ├── cli/        # myteams_cli: interactive client
    │   ├── server/     # myteams_server: event-loop server
    │   └── common/     # shared data model, protocol, utils
    ├── docs/           # RFC-style protocol & data model documentation (Typst)
    ├── libs/           # native libraries (libmyteams)
    ├── Makefile
    └── Cargo.toml

## Requirements

- Rust toolchain (edition 2024)
- `make`
- The native `libmyteams` library placed under `libs/myteams/`
  (see `LDFLAGS` in the Makefile)
- Optional: `typst`, `pandoc` to regenerate the docs

## Build

    make            # builds both binaries and copies them to the project root
    make build      # same as above
    make re         # clean rebuild
    make clean      # cargo clean
    make fclean     # clean + remove generated docs

The build links against `libmyteams`:

    RUSTFLAGS="-L $(pwd)/libs/myteams -l myteams -C rpath=no" cargo build --bin myteams_server
    RUSTFLAGS="-L $(pwd)/libs/myteams -l myteams -C rpath=no" cargo build --bin myteams_cli

## Usage

Start the server first, then connect with the client:

    ./myteams_server <port>
    ./myteams_cli <host> <port>

### Client commands

    /help
    /login ["user_name"]
    /logout
    /users
    /user ["user_uuid"]
    /send ["user_uuid"] ["message_body"]
    /messages ["user_uuid"]
    /subscribe ["team_uuid"]
    /subscribed ?["team_uuid"]
    /unsubscribe ["team_uuid"]
    /use ?["team_uuid"] ?["channel_uuid"] ?["thread_uuid"]
    /create
    /list
    /info

## Data model

Global limits (enforced everywhere):

| Field | Max length |
| --- | --- |
| Name | 32 |
| Description | 255 |
| Body | 512 |

Entities:

- **User** — UUID, username (≤ 32 chars), online/offline status
- **Team** — UUID, name, description, subscribers (list of user UUIDs)
- **Channel** (parent: Team) — UUID, name, description
- **Thread** (parent: Channel) — UUID, title, message, author, timestamp
- **Comment** (parent: Thread) — body, author

## Protocol

Messages follow a simple text format over TCP:

    <COMMAND> <arg1> <arg2> ... \n

Responses use standard status codes:

- `2xx` — success
- `4xx/5xx` — error
- Async events are pushed to subscribed clients

See [`src/common/protocol/`](src/common/protocol/) and [`DOCS.md`](DOCS.md) for the
full RFC-style specification.

## Documentation

The `docs/` directory contains an RFC-style specification written in Typst:

    make docs   # compiles docs/main.typ into pdf/md/txt/html
