use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Api {
    pub port: u16,
}

impl Api {}
