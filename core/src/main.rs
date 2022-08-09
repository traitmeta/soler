use axum::{
    body,
    middleware,
    routing::{get, post},
    Router,
};
use core::{print, validater, serve};
use tower::ServiceBuilder;
use tower_http::ServiceBuilderExt;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "core=debug".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();
    let app = Router::new()
        .route(
            "/",
            post(print::handler).layer(
                ServiceBuilder::new()
                    .map_request_body(body::boxed)
                    .layer(middleware::from_fn(print::print_request_body)),
            ),
        )
        .route("/name", get(validater::handler));

    serve::Serve(app, 3000).await;
}
