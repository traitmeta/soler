use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Chain {
    pub url: String,
    pub chain_id: Option<u64>,
    pub chain_name: String,
    pub contracts: Option<Vec<String>>,
}

impl Chain {
    
}
