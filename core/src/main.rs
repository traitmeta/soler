use axum::{
    body,
    http::{HeaderValue, Method},
    middleware,
    response::{Html, IntoResponse},
    routing::{get, post},
    Json, Router,
};
use core::{print, validater};
use std::net::SocketAddr;
use tower::ServiceBuilder;
use tower_http::{cors::CorsLayer, ServiceBuilderExt};
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

    serve(app, 3000).await;
}

async fn serve(app: Router, port: u16) {
    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    tracing::debug!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn cors() {
    let frontend = async {
        let app = Router::new().route("/", get(html));
        serve(app, 3000).await;
    };

    let backend = async {
        let app = Router::new().route("/json", get(json)).layer(
            // see https://docs.rs/tower-http/latest/tower_http/cors/index.html
            // for more details
            //
            // pay attention that for some request types like posting content-type: application/json
            // it is required to add ".allow_headers([http::header::CONTENT_TYPE])"
            // or see this issue https://github.com/tokio-rs/axum/issues/849
            CorsLayer::new()
                .allow_origin("http://localhost:3000".parse::<HeaderValue>().unwrap())
                .allow_methods([Method::GET]),
        );
        serve(app, 4000).await;
    };

    tokio::join!(frontend, backend);
}

async fn html() -> impl IntoResponse {
    Html(
        r#"
        <script>
            fetch('http://localhost:3000/json')
              .then(response => response.json())
              .then(data => console.log(data));
        </script>
        "#,
    )
}

async fn json() -> impl IntoResponse {
    Json(vec!["one", "two", "three"])
}
