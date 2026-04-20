use crate::common::utils::serializing::{read_length, read_str, write_length, write_str};
use crate::common::utils::uuid_v4;
use crate::common::Thread;
use std::collections::HashMap;
use std::fs::File;

#[derive(Clone)]
pub struct Channel {
    pub uuid: String,
    pub name: String,
    pub description: String,
    pub threads: HashMap<String, Thread>,
}
impl Channel {
    pub fn new(name: &str) -> Self {
        Channel {
            uuid: uuid_v4(),
            name: String::from(name),
            description: String::new(),
            threads: HashMap::new(),
        }
    }

    pub fn write_to_file(&self, file: &mut File) {
        write_str(file, &self.uuid).unwrap();
        write_str(file, &self.name).unwrap();
        write_str(file, &self.description).unwrap();

        write_length(file, self.threads.len() as u64).unwrap();
        for thread in &self.threads {
            thread.1.write_to_file(file);
        }
    }

    pub fn read_from_file(file: &mut File) -> Channel {
        let uuid = read_str(file).unwrap();
        let name = read_str(file).unwrap();
        let description = read_str(file).unwrap();
        let mut channel = Channel {
            uuid,
            name,
            description,
            threads: HashMap::new(),
        };

        let messages_length = read_length(file).unwrap();
        for _ in 0..messages_length {
            let thread = Thread::read_from_file(file);
            channel.threads.insert(thread.uuid.clone(), thread);
        }
        channel
    }
}
