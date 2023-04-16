use serde::{Deserialize, Serialize};

use crate::{chain::Chain, db::DB, kafka::Kafka, redis::Redis, Config};

#[derive(Clone, Debug, Deserialize, Serialize, Default)]
#[serde(rename_all = "kebab-case")]
pub struct BaseConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub database: Option<DB>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub redis: Option<Redis>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kafka: Option<Kafka>,
    /// SeedPeers configured with a PeerId are preferred and the node will always try to ensure a
    /// connection is established with these nodes.
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub chains: Vec<Chain>,
}

impl Config for BaseConfig {}
