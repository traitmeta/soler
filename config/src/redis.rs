use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Redis {
    pub url: String,
    pub username: String,
    pub password: String,
}

// redis://:password@ip:port/

impl Redis {
    pub fn url(&self) -> String {
        format!("redis://{}:{}@{}/", self.username, self.password, self.url,)
    }
}
