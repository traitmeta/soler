use anyhow::Result;
use clap::Parser;
use config::Config;
use config::{base::BaseConfig, Args};
use std::time::Duration;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

const GIT_REVISION: &str = {
    if let Some(revision) = option_env!("GIT_REVISION") {
        revision
    } else {
        let version = git_version::git_version!(
            args = ["--always", "--dirty", "--exclude", "*"],
            fallback = ""
        );

        if version.is_empty() {
            panic!("unable to query git revision");
        }
        version
    }
};
const VERSION: &str = const_str::concat!(env!("CARGO_PKG_VERSION"), "-", GIT_REVISION);

// cargo run --package config  -- --config-path "example.yaml"
#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "config=debug".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let args = Args::parse();
    let config = BaseConfig::load(&args.config_path)?;
    info!("config info {:?}", config);
    info!("config info {:?}", config.database);

    assert!(
        config.database.is_some(),
        "database cannot be read from the config file"
    );

    info!("Git version: {VERSION}");
    if let Some(kafka) = config.kafka {
        info!("Started Kafka endpoint at {:?}", kafka.brokers);
    }

    info!("Started Chains lens {:?}", config.chain);

    // TODO: Do we want to provide a way for the node to gracefully shutdown?
    loop {
        tokio::time::sleep(Duration::from_secs(1000)).await;
    }
}
