use api::{handlers::state, router};
use clap::Parser;
use config::{base::BaseConfig, Args, Config};
use repo::orm::conn::connect_db;
use std::net::SocketAddr;
use tracing::info;
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt};

// RUST_LOG=debug cargo run --package api
#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "app=debug".into()),
        ))
        .with(fmt::layer())
        .init();

    let args = Args::parse();
    let config = BaseConfig::load(&args.config_path).unwrap();
    let api = config.api.unwrap();
    let addr = SocketAddr::from(([0, 0, 0, 0], api.port));
    info!(message = "listening", addr = ?addr);

    let db_cfg = config.database.unwrap();
    info!(message = "db config", cfg = ?db_cfg);

    let conn = connect_db(db_cfg).await.unwrap();
    info!(message = "connected db");

    router::route(addr, state::AppState { conn }).await
}
