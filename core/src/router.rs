use crate::handlers::{common, helth, user};
use axum::{
    body, middleware,
    routing::{get, post},
    Router,
};
use std::net::SocketAddr;
use tokio::signal;
use tower::ServiceBuilder;
use tower_http::ServiceBuilderExt;

use super::{
    auth::jwt,
    handlers::{body_parser, err, form, response},
};

pub async fn route(addr: SocketAddr) {
    // build our application with a route
    let app = Router::new()
        .route("/", get(helth::home))
        .route("/info", get(helth::info))
        .route("/user/create", post(user::create_user))
        .route("/err", get(err::handler))
        .route("/form", get(form::show_form).post(form::accept_form))
        .route("/user-info", post(form::accept_form))
        .route("/protected", get(jwt::protected))
        .route("/authorize/bearer", post(jwt::authorize))
        .route("/authorize/api", post(jwt::authorize_api_token))
        .layer(middleware::from_fn(response::print_request_response));

    // add a fallback service for handling routes to unknown paths
    let app = app.fallback(common::handler_404);

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
