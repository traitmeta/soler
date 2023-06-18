use config::db::DB;
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
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    tracing::debug!("listening on {}", addr);

    let db_cfg = DB {
        url: "localhost:3306".to_string(),
        schema: "mysql".to_string(),
        username: "root".to_string(),
        password: "meta".to_string(),
        database: "rust_test".to_string(),
    };
    let conn = connect_db(db_cfg).await.unwrap();

    router::route(addr, state::AppState { conn }).await
}
