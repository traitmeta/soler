use clap::Parser;
use config::{base::BaseConfig, Args, Config};
use repo::orm::conn::connect_db;
use scanner::{
    evms::eth::EthCli,
    handler::block::init_block,
    tasks::{block::sync_task, token::strat_token_metadata_task},
};
use std::sync::Arc;
use tracing::instrument;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

// RUST_LOG=debug cargo run --package scanner
#[tokio::main]
#[instrument]
async fn main() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "scanner=debug".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let args = Args::parse();
    let config = BaseConfig::load(&args.config_path).unwrap();
    let mut goerli_url = "";
    if config.chains.contains_key("Goerli") {
        let goerli_chain_cfg = config.chains.get("Goerli").unwrap();
        goerli_url = goerli_chain_cfg.url.as_str();
    } else {
        tracing::error!("Init failed for not found Goerli chain config");
    }

    let rpc_url = Arc::new(goerli_url.to_string());
    let db_cfg = config.database.unwrap();
    let conn = connect_db(db_cfg.clone()).await.unwrap();
    let eth_cli = EthCli::new(rpc_url.clone().as_str());
    init_block(eth_cli, &conn).await;

    let sync_db_cfg = db_cfg.clone();
    let block_task = tokio::spawn(sync_task(rpc_url.clone().to_string(), sync_db_cfg));

    tracing::debug!(
        "end chain sync: {:?}",
        config.chains.get("Goerli").unwrap().chain_name
    );

    // strat_token_metadata_task(goerli_url, &conn).await;
    let sync_db_cfg = db_cfg.clone();
    let token_task = tokio::spawn(strat_token_metadata_task(
        rpc_url.clone().to_string(),
        sync_db_cfg,
    ));

    block_task.await.unwrap();
    token_task.await.unwrap();
}
