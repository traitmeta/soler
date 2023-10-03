use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Kafka {
    pub brokers: Vec<String>, // localhost:port,localhost:port
    pub topics: Vec<String>,  // test1,test2
    pub group_id: Option<String>,
    pub log_level: Option<String>,
}

impl Kafka {
    // pub fn topics_to_vec(&self) -> Vec<&str> {
    //     self.topics.split(',').collect()
    // }

    pub fn brokers_to_str(&self) -> String {
        self.brokers.join(",")
    }
}
