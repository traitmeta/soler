use hyper::body::Bytes;
use jsonrpsee::core::client::{ClientT, Subscription};
use jsonrpsee::http_client::HttpClientBuilder;
use jsonrpsee::rpc_params;
use jsonrpsee::ws_client::WsClientBuilder;
use rpc::RpcClient;
use rpc::{server, server::ExampleHash, server::ExampleStorageKey};
use std::time::Duration;
use tower_http::trace::{DefaultMakeSpan, DefaultOnResponse, TraceLayer};
use tower_http::LatencyUnit;
use tracing::Level;
use tracing_subscriber::util::SubscriberInitExt;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let filter = tracing_subscriber::EnvFilter::from_default_env();
    tracing_subscriber::FmtSubscriber::builder()
        .with_env_filter(filter)
        .with_max_level(Level::TRACE)
        .finish()
        .try_init()
        .expect("setting default subscriber failed");

    test_with_run_ws().await;

    Ok(())
}

async fn test_with_run_ws() {
    let ws_server_addr = server::run_simple_ws_server().await.unwrap();
    let ws_url = format!("ws://{}", ws_server_addr);
    let ws_client = WsClientBuilder::default().build(&ws_url).await.unwrap();
    let response: String = ws_client.request("say_hello", rpc_params![]).await.unwrap();
    tracing::info!("response: {:?}", response);

    let storage_res = ws_client
        .storage_keys(vec![1, 2, 3, 4], None::<ExampleHash>)
        .await
        .unwrap();
    assert_eq!(storage_res, vec![vec![1, 2, 3, 4]]);

    let mut sub: Subscription<Vec<ExampleHash>> =
        RpcClient::<ExampleHash, ExampleStorageKey>::subscribe_storage(&ws_client, None)
            .await
            .unwrap();
    assert_eq!(Some(vec![[0; 32]]), sub.next().await.transpose().unwrap());
}

async fn test_with_run_http() {
    let http_server_addr = server::run_simple_http_server().await.unwrap();
    let http_url = format!("http://{}", http_server_addr);
    let middleware = tower::ServiceBuilder::new()
	.layer(
		TraceLayer::new_for_http()
			.on_request(
				|request: &hyper::Request<hyper::Body>, _span: &tracing::Span| tracing::info!(request = ?request, "on_request"),
			)
			.on_body_chunk(|chunk: &Bytes, latency: Duration, _: &tracing::Span| {
				tracing::info!(size_bytes = chunk.len(), latency = ?latency, "sending body chunk")
			})
			.make_span_with(DefaultMakeSpan::new().include_headers(true))
			.on_response(DefaultOnResponse::new().include_headers(true).latency_unit(LatencyUnit::Micros)),
	);

    let client = HttpClientBuilder::default()
        .set_middleware(middleware)
        .build(http_url)
        .unwrap();
    let params = rpc_params![1_u64, 2, 3];
    let response: Result<String, _> = client.request("say_hello", params).await;
    tracing::info!("r: {:?}", response);
}
