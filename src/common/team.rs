use crate::common::Channel;
use crate::common::utils::serializing::{read_length, read_str, write_length, write_str};
use crate::common::utils::uuid_v4;
use std::collections::HashMap;
use std::fs::File;

pub struct Team {
    pub uuid: String,
    pub name: String,
    pub description: String,
    pub channels: HashMap<String, Channel>,
    pub users_uuid: Vec<String>,
}

impl Team {
    pub fn new(name: String) -> Self {
        Team {
            uuid: uuid_v4(),
            name,
            description: String::new(),
            channels: HashMap::new(),
            users_uuid: Vec::new(),
        }
    }

    pub fn write_to_file(&self, file: &mut File) {
        write_str(file, self.uuid.as_str()).unwrap();
        write_str(file, self.name.as_str()).unwrap();
        write_str(file, self.description.as_str()).unwrap();

        write_length(file, self.channels.len() as u64).expect("Error saving channels");
        for channel in &self.channels {
            channel.1.write_to_file(file);
        }

        write_length(file, self.users_uuid.len() as u64).unwrap();
        for uuid in &self.users_uuid {
            write_str(file, uuid).unwrap()
        }
    }

    pub fn read_from_file(file: &mut File) -> Team {
        let uuid = read_str(file).unwrap();
        let name = read_str(file).unwrap();
        let description = read_str(file).unwrap();
        let mut team = Team {
            uuid,
            name,
            description,
            channels: HashMap::new(),
            users_uuid: Vec::new(),
        };

        let channels_length = read_length(file).unwrap();
        for _ in 0..channels_length {
            let channel = Channel::read_from_file(file);
            team.channels.insert(channel.uuid.clone(), channel);
        }

        let users_length = read_length(file).unwrap();
        for _ in 0..users_length {
            team.users_uuid.push(read_str(file).unwrap());
        }

        team
    }
}
