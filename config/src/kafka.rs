use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Kafka {
    pub brokers: Vec<String>,
    pub topic: String,
    pub group_id: String,
}

impl Kafka {
   
}
