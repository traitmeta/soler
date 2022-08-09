use axum::{
    http::{HeaderValue, Method},
    response::{Html, IntoResponse},
    routing::get,
    Json, Router,
};
use tower_http::{cors::CorsLayer};

use crate::serve;

async fn cors() {
    let frontend = async {
        let app = Router::new().route("/", get(html));
        serve::Serve(app, 3000).await;
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
        serve::Serve(app, 4000).await;
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