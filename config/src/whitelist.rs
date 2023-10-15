use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Addr {
    pub address: String,
}

impl Addr {}
