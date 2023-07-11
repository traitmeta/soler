pub mod server;

use jsonrpsee::core::SubscriptionResult;
use jsonrpsee::proc_macros::rpc;
use jsonrpsee::types::ErrorObjectOwned;

#[rpc(server, client, namespace = "state")]
pub trait Rpc<Hash, StorageKey>
where
    Hash: std::fmt::Debug,
{
    // Async method call example.
    #[method(name = "getKeys")]
    async fn storage_keys(
        &self,
        storage_key: StorageKey,
        hash: Option<Hash>,
    ) -> Result<Vec<StorageKey>, ErrorObjectOwned>;

    // Subscription that takes a `StorageKey` as input and produces a `Vec<Hash>`.
    #[subscription(name = "subscribeStorage" => "override", item = Vec<Hash>)]
    async fn subscribe_storage(&self, keys: Option<Vec<StorageKey>>) -> SubscriptionResult;
}
