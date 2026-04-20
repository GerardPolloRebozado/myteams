use crate::common::utils::serializing::{read_str, write_str};
use crate::common::utils::uuid_v4;
use std::fs::File;
use crate::server_event_user_loaded;

#[derive(Clone)]
pub struct User {
    pub uuid: String,
    pub name: String,
}

impl User {
    pub fn new(name: String) -> Self {
        Self {
            uuid: uuid_v4(),
            name,
        }
    }

    pub fn from_string(s: &str) -> Option<Self> {
        let parts: Vec<&str> = s.splitn(2, ' ').collect();
        if parts.len() != 2 {
            return None;
        }
        Some(Self {
            uuid: parts[0].to_string(),
            name: parts[1].to_string(),
        })
    }

    pub fn to_string(&self) -> String {
        format!("{} {}", self.uuid, self.name)
    }

    pub fn write_to_file(&self, file: &mut File) {
        write_str(file, &self.to_string()).unwrap();
    }

    pub fn read_from_file(file: &mut File) -> User {
        let msg= read_str(file).unwrap();
        let user = User::from_string(&msg).expect("Error importing message from file");
        server_event_user_loaded(user.uuid.as_str(), user.name.as_str());
        user
    }
}
