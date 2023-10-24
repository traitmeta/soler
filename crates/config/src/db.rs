use serde::{Deserialize, Serialize};
use tracing::log::LevelFilter;
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct DB {
    pub url: String,
    pub schema: String,
    pub username: String,
    pub password: String,
    pub database: String,
    pub log_level: u8,
}

// mysql://root:meta@localhost/test

impl DB {
    pub fn url(&self) -> String {
        format!(
            "{}://{}:{}@{}/{}",
            self.schema, self.username, self.password, self.url, self.database
        )
    }
    pub fn from_usize(&self) -> LevelFilter {
        match self.log_level {
            0 => LevelFilter::Off,
            1 => LevelFilter::Error,
            2 => LevelFilter::Warn,
            3 => LevelFilter::Info,
            4 => LevelFilter::Debug,
            5 => LevelFilter::Trace,
            _ => LevelFilter::Debug,
        }
    }
}
