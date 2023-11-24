use axum::{
    middleware,
    routing::{get, post},
    Extension, Router,
};
use std::{net::SocketAddr, sync::Arc};
use tokio::signal;

use crate::{
    biz::{address, token_transfer},
    err,
};

use super::{
    auth::jwt,
    biz::{block, event, helth, response, state, transaction, user},
};

pub async fn route(addr: SocketAddr, app_state: state::AppState) {
    // build our application with a route
    let app = Router::new()
        .route("/", get(helth::home))
        .route("/info", get(helth::info))
        .route("/user/create", post(user::create_user))
        .route("/block/:id", get(block::get_block))
        .route("/txs", post(transaction::gets_transaction))
        .route("/tx/:id", get(transaction::get_transaction))
        .route("/tx/:id/logs", get(event::get_transaction_logs))
        .route(
            "/tx/:id/token-transfers",
            get(token_transfer::get_token_transfers),
        )
        .route("/address/:id", get(transaction::get_transaction))
        .route("/address/tokens", get(address::get_address_tokens))
        .route("/protected", get(jwt::protected))
        .route("/authorize/bearer", post(jwt::authorize))
        .route("/authorize/api", post(jwt::authorize_api_token))
        .layer(middleware::from_fn(response::print_request_response))
        .layer(Extension(Arc::new(app_state)));

    // add a fallback service for handling routes to unknown paths
    let app = app.fallback(err::handler_404);

    // run it
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .with_graceful_shutdown(shutdown_signal())
        .await
        .unwrap();
}

pub async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    println!("signal received, starting graceful shutdown");
}
