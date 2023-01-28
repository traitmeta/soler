use crate::handlers::{common, user};
use axum::{
    body,
    handler::Handler,
    middleware,
    routing::{get, post},
    Router,
};
use std::net::SocketAddr;
use tokio::signal;
use tower::ServiceBuilder;
use tower_http::ServiceBuilderExt;

use super::{handlers::{body_parser, err, form, response}, auth::jwt};

pub async fn route(addr: SocketAddr) {
    // build our application with a route
    let app = Router::new()
        .route("/", get(user::handler))
        .route("/user", get(user::root))
        .route("/user/create", post(user::create_user))
        .route("/anyhow", get(err::handler))
        .route("/foo", post(body_parser::handler))
        .route("/form", get(form::show_form).post(form::accept_form))
        .route("/protected", get(jwt::protected))
        .route("/authorize/bearer", post(jwt::authorize))
        .route("/authorize/api", post(jwt::authorize_api_token))
        .layer(
            ServiceBuilder::new()
                .map_request_body(body::boxed)
                .layer(middleware::from_fn(body_parser::print_request_body)),
        )
        .layer(middleware::from_fn(response::print_request_response));

    // add a fallback service for handling routes to unknown paths
    let app = app.fallback(common::handler_404.into_service());

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
