use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Kafka {
    pub brokers: Vec<String>,
    pub topic: String,
    pub group_id: Option<String>,
}

impl Kafka {
   
}
