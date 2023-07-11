use crate::RpcServer;
use jsonrpsee::core::{async_trait, SubscriptionResult};
use jsonrpsee::server::ServerBuilder;
use jsonrpsee::server::{PendingSubscriptionSink, SubscriptionMessage};
use jsonrpsee::types::ErrorObjectOwned;
use jsonrpsee::RpcModule;
use std::net::SocketAddr;

pub type ExampleHash = [u8; 32];
pub type ExampleStorageKey = Vec<u8>;

pub struct RpcServerImpl;

#[async_trait]
impl RpcServer<ExampleHash, ExampleStorageKey> for RpcServerImpl {
    async fn storage_keys(
        &self,
        storage_key: ExampleStorageKey,
        _hash: Option<ExampleHash>,
    ) -> Result<Vec<ExampleStorageKey>, ErrorObjectOwned> {
        Ok(vec![storage_key])
    }

    // Note that the server's subscription method must return `SubscriptionResult`.
    async fn subscribe_storage(
        &self,
        pending: PendingSubscriptionSink,
        _keys: Option<Vec<ExampleStorageKey>>,
    ) -> SubscriptionResult {
        let sink = pending.accept().await?;
        let msg = SubscriptionMessage::from_json(&vec![[0; 32]])?;
        sink.send(msg).await?;

        Ok(())
    }
}

pub async fn run_simple_ws_server() -> anyhow::Result<SocketAddr> {
    let server = ServerBuilder::default()
        .build("127.0.0.1:0".parse::<SocketAddr>()?)
        .await?;
    let mut module = RpcModule::new(());
    module.register_method("say_hello", |_, _| "lo")?;
    module.merge(RpcServerImpl.into_rpc()).unwrap();

    let addr = server.local_addr()?;
    let handle = server.start(module)?;

    // In this example we don't care about doing shutdown so let's it run forever.
    // You may use the `ServerHandle` to shut it down or manage it yourself.
    tokio::spawn(handle.stopped());
    Ok(addr)
}

pub async fn run_simple_http_server() -> anyhow::Result<SocketAddr> {
    let server = ServerBuilder::default().build("127.0.0.1:0").await?;
    let mut module = RpcModule::new(());
    module.register_method("say_hello", |_, _| "lo")?;
    let addr = server.local_addr()?;
    let handle = server.start(module)?;

    // In this example we don't care about doing shutdown so let's it run forever.
    // You may use the `ServerHandle` to shut it down or manage it yourself.
    tokio::spawn(handle.stopped());

    Ok(addr)
}
