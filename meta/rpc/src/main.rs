use std::net::SocketAddr;

use jsonrpsee::core::client::{Subscription, ClientT};
use jsonrpsee::http_client::HttpClientBuilder;
use jsonrpsee::http_server::{HttpServerBuilder, HttpServerHandle};
use jsonrpsee::{rpc_params, RpcModule};
use jsonrpsee::ws_client::WsClientBuilder;
use jsonrpsee::ws_server::{WsServerBuilder, WsServerHandle};
use rpc::RpcServerImpl;
use rpc::{ExampleHash, RpcClient};
use rpc::{ExampleStorageKey, RpcServer};
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

    let (ws_server_addr, _handle) = ws_run_server().await?;
    let ws_url = format!("ws://{}", ws_server_addr);
    let ws_client = WsClientBuilder::default().build(&ws_url).await?;

    let (http_server_addr, _handle) = http_run_server().await?;
    let http_url = format!("http://{}", http_server_addr);
	let http_client = HttpClientBuilder::default().build(http_url)?;

    assert_eq!(
        ws_client
            .storage_keys(vec![1, 2, 3, 4], None::<ExampleHash>)
            .await
            .unwrap(),
        vec![vec![1, 2, 3, 4]]
    );

    let mut sub: Subscription<Vec<ExampleHash>> =
        RpcClient::<ExampleHash, ExampleStorageKey>::subscribe_storage(&ws_client, None)
            .await
            .unwrap();
    assert_eq!(Some(vec![[0; 32]]), sub.next().await.transpose().unwrap());

 
	let params = rpc_params!(1_u64, 2, 3);
	let response: Result<String, _> = http_client.request("say_hello", params).await;
	tracing::info!("r: {:?}", response);

    Ok(())
}

async fn ws_run_server() -> anyhow::Result<(SocketAddr, WsServerHandle)> {
    let server = WsServerBuilder::default().build("127.0.0.1:0").await?;
    
    let addr = server.local_addr()?;
    let handle = server.start(RpcServerImpl.into_rpc())?;
    Ok((addr, handle))
}

async fn http_run_server() -> anyhow::Result<(SocketAddr, HttpServerHandle)> {
    let server = HttpServerBuilder::default().build("127.0.0.1:0".parse::<SocketAddr>()?).await?;
	let mut module = RpcModule::new(());
	module.register_method("say_hello", |_, _| Ok("lo"))?;

	let addr = server.local_addr()?;
	let server_handle = server.start(module)?;
	Ok((addr, server_handle))
}
