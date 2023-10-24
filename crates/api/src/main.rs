use api::{biz::state, router};
use clap::Parser;
use config::{base::BaseConfig, Args, Config};
use repo::orm::conn::connect_db;
use std::net::SocketAddr;
use tracing::info;
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

// RUST_LOG=debug cargo run --package api
#[tokio::main]
async fn main() {
    let file_appender = tracing_appender::rolling::daily("logs", "api.log");
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    let formatting_layer = fmt::layer().pretty().with_writer(std::io::stderr);
    let file_layer = fmt::layer().with_ansi(false).with_writer(non_blocking);
    tracing_subscriber::Registry::default()
        .with(env_filter)
        .with(formatting_layer)
        .with(file_layer)
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
