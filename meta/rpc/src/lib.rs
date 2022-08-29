use jsonrpsee::core::{async_trait, Error};
use jsonrpsee::proc_macros::rpc;
use jsonrpsee::types::SubscriptionResult;
use jsonrpsee::ws_server::SubscriptionSink;

pub type ExampleHash = [u8; 32];
pub type ExampleStorageKey = Vec<u8>;

#[rpc(server, client, namespace = "state")]
pub trait Rpc<Hash: Clone, StorageKey>
where
    Hash: std::fmt::Debug,
{
    /// Async method call example.
    #[method(name = "getKeys")]
    async fn storage_keys(
        &self,
        storage_key: StorageKey,
        hash: Option<Hash>,
    ) -> Result<Vec<StorageKey>, Error>;

    /// Subscription that takes a `StorageKey` as input and produces a `Vec<Hash>`.
    #[subscription(name = "subscribeStorage" => "override", item = Vec<Hash>)]
    fn subscribe_storage(&self, keys: Option<Vec<StorageKey>>);
}

pub struct RpcServerImpl;

#[async_trait]
impl RpcServer<ExampleHash, ExampleStorageKey> for RpcServerImpl {
    async fn storage_keys(
        &self,
        storage_key: ExampleStorageKey,
        _hash: Option<ExampleHash>,
    ) -> Result<Vec<ExampleStorageKey>, Error> {
        Ok(vec![storage_key])
    }

    // Note that the server's subscription method must return `SubscriptionResult`.
    fn subscribe_storage(
        &self,
        mut sink: SubscriptionSink,
        _keys: Option<Vec<ExampleStorageKey>>,
    ) -> SubscriptionResult {
        let _ = sink.send(&vec![[0; 32]]);
        Ok(())
    }
}
