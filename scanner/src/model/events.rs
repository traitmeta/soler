use serde::{Deserialize, Serialize};

// 这里没有考虑tx_index
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Event {
    pub height: u64,
    pub timestamp_ms: u64,
    pub tx_id: String,
    pub contract_adress: String,
    pub event_name: String,
    pub event_content: String,
}
