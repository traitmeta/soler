use clap::Parser;
use config::{base::BaseConfig, Args, Config};
use repo::orm::conn::connect_db;
use scanner::{
    evms::eth::EthCli,
    handler::block::EthHandler,
};

use tracing::instrument;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};


// RUST_LOG=debug cargo run --package scanner
#[tokio::main]
#[instrument]
async fn main() -> web3::Result<()> {
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

    let db_cfg = config.database.unwrap();
    let conn = connect_db(db_cfg).await.unwrap();
    let eth_cli = EthCli::new(goerli_url);
    let eth_handler = EthHandler::new(eth_cli, conn);
    eth_handler.init_block().await;

    tracing::debug!(
        "end chain sync: {:?}",
        config.chains.get("Goerli").unwrap().chain_name
    );
    
    // let mut height = 0;
    // if let Some(db_height) = log_scanner_current_height(&conn, "eth:5", "eth").await {
    //     height = db_height + 1;
    // }

    // let res = Mutation::update_height_by_task_name(&conn, "eth:5", height).await;
    // match res {
    //     Err(e) => tracing::debug!("hanlder height {} failed,err:{}", height, e),
    //     _ => tracing::debug!("hanlded height: {}", height),
    // }

    Ok(())
}
