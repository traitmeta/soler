use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct DB {
    pub url: String,
    pub schema: String,
    pub username: String,
    pub password: String,
    pub database: String,
}

// mysql://root:meta@localhost/test

impl DB {
    pub fn url(&self) -> String {
        format!(
            "{}://{}:{}@{}/{}",
            self.schema, self.username, self.password, self.url, self.database
        )
    }
}
