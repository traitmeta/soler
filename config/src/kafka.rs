use std::string;

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Kafka {
    pub brokers: String, // localhost:port,localhost:port
    pub topics: String,  // test1,test2
    pub group_id: Option<String>,
    pub log_level: String,
}

impl Kafka {
    pub fn topics_to_vec(&self) -> Vec<&str> {
        self.topics.split(',').collect()
    }
}
