use clap::Parser;
use config::{base::BaseConfig, Args, Config};
use core::{handlers::state, router};
use repo::orm::conn::connect_db;
use std::net::SocketAddr;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "core=debug".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    // run it
    let addr = SocketAddr::from(([0, 0, 0, 0], 50060));
    tracing::debug!("listening on {}", addr);

    let args = Args::parse();
    let config = BaseConfig::load(&args.config_path).unwrap();
    let db_cfg = config.database.unwrap();
    tracing::info!("db config {:?}", db_cfg);
    let conn = connect_db(db_cfg).await.unwrap();
    tracing::info!("connected db");

    router::route(addr, state::AppState { conn }).await
}
