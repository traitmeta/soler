use jsonrpsee::core::{async_trait, SubscriptionResult};
use jsonrpsee::server::{PendingSubscriptionSink, SubscriptionMessage};
use jsonrpsee::types::ErrorObjectOwned;

use crate::RpcServer;

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
