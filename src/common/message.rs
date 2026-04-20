use crate::common::date::Date;
use crate::common::utils::serializing::{read_str, write_str};
use crate::common::utils::uuid_v4;
use std::fs::File;

#[derive(Clone)]
pub enum Location {
    THREAD,
    USER,
}

impl Location {
    pub fn to_string(&self) -> String {
        match self {
            Location::THREAD => "THREAD".to_string(),
            Location::USER => "USER".to_string(),
        }
    }

    pub fn from_string(s: &str) -> Option<Location> {
        match s {
            "THREAD" => Some(Location::THREAD),
            "USER" => Some(Location::USER),
            _ => None,
        }
    }
}

#[derive(Clone)]
pub struct Message {
    pub uuid: String,
    pub message: String,
    pub sender_uuid: String,
    pub date: Date,
    pub location_type: Location,
    pub location_uuid: String,
}

impl Message {
    pub fn new(
        sender_uuid: String,
        location_type: Location,
        location_uuid: String,
        message: String,
    ) -> Message {
        Message {
            uuid: uuid_v4(),
            message,
            sender_uuid,
            date: Date::now(),
            location_type,
            location_uuid,
        }
    }

    pub fn write_to_file(&self, file: &mut File) {
        write_str(file, self.to_string().as_str()).unwrap();
    }

    pub fn read_from_file(file: &mut File) -> Message {
        let msg= read_str(file).unwrap();
        Message::from_string(&msg).expect("Error importing message from file")
    }

    pub fn to_string(&self) -> String {
        format!(
            "{}|{}|{}|{}|{}|{}",
            self.uuid,
            self.sender_uuid,
            self.date.to_string(),
            self.location_type.to_string(),
            self.location_uuid,
            self.message
        )
    }

    pub fn from_string(message: &str) -> Option<Message> {
        let parts: Vec<&str> = message.splitn(6, '|').collect();

        if parts.len() != 6 {
            return None;
        }

        let uuid = parts[0].to_string();
        let sender = parts[1].to_string();
        let date = Date::from_string(parts[2])?;
        let location_type = Location::from_string(parts[3])?;
        let location_uuid = parts[4].to_string();
        let msg = parts[5].to_string();

        Some(Message {
            uuid,
            message: msg,
            sender_uuid: sender,
            date,
            location_type,
            location_uuid,
        })
    }
}
