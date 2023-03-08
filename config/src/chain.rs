use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Chain {
    pub url: String,
    pub chain_id: u64,
    pub chain_name: String,
    pub contracts: Vec<String>,
}

impl Chain {
    
}
