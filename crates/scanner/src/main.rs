use clap::Parser;
use config::{base::BaseConfig, Args, Config};
use repo::orm::conn::connect_db;
use scanner::{
    contracts::erc20::IERC20Call,
    evms::eth::EthCli,
    handler::block::init_block,
    tasks::{block::handle_block_task, token::token_metadata_task},
};
use std::sync::Arc;
use tracing::instrument;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

// RUST_LOG=debug cargo run --package scanner
#[instrument]
fn main() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "scanner=debug".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let args = Args::parse();
    let config = BaseConfig::load(args.config_path).unwrap();
    let mut goerli_url = "";
    if config.chains.contains_key("Goerli") {
        let goerli_chain_cfg = config.chains.get("Goerli").unwrap();
        goerli_url = goerli_chain_cfg.url.as_str();
    } else {
        tracing::error!("Init failed for not found Goerli chain config");
    }

    let rpc_url = Arc::new(goerli_url.to_string());
    let db_cfg = config.database.unwrap();
    let scanner = tokio::runtime::Builder::new_multi_thread()
        .thread_name("scanner-runtime")
        .enable_all()
        .build()
        .unwrap();

    scanner.spawn(async move {
        let conn = connect_db(db_cfg.clone()).await.unwrap();
        let conn = Arc::new(conn);
        let eth_cli = EthCli::new(rpc_url.clone().as_str());
        let eth_cli = Arc::new(eth_cli);
        init_block(eth_cli.clone(), conn.clone()).await;

        handle_block_task(eth_cli.clone(), conn.clone());
        tracing::debug!(
            "end chain sync: {:?}",
            config.chains.get("Goerli").unwrap().chain_name
        );

        let erc20_call = Arc::new(IERC20Call::new(rpc_url.as_str()));
        token_metadata_task(erc20_call.clone(), conn.clone());
        token_metadata_task(erc20_call.clone(), conn.clone());
    });

    // wait for SIGINT on the main thread
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(wait_termination());

    drop(scanner);
}

#[cfg(not(unix))]
// On windows we wait for whatever "ctrl_c" means there
async fn wait_termination() {
    tokio::signal::ctrl_c().await.unwrap()
}

#[cfg(unix)]
// On unix we wait for both SIGINT (when run in terminal) and SIGTERM(when run in docker or other supervisor)
// Docker stop sends SIGTERM: https://www.baeldung.com/ops/docker-stop-vs-kill#:~:text=The%20docker%20stop%20commands%20issue,rather%20than%20killing%20it%20immediately.
// Systemd by default sends SIGTERM as well: https://www.freedesktop.org/software/systemd/man/systemd.kill.html
// Upstart also sends SIGTERM by default: https://upstart.ubuntu.com/cookbook/#kill-signal
async fn wait_termination() {
    use futures::future::select;
    use futures::FutureExt;
    use tokio::signal::unix::*;

    let sigint = tokio::signal::ctrl_c().map(Result::ok).boxed();
    let mut sigterm = signal(SignalKind::terminate()).unwrap();
    let sigterm_recv = sigterm.recv().boxed();
    select(sigint, sigterm_recv).await;
}
