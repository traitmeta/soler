use futures::stream;
use futures::StreamExt;
use futures_core::Stream;
use sui_json_rpc_types::SuiTransactionBlockResponseOptions;
use sui_json_rpc_types::{SuiTransactionBlockResponse, SuiTransactionBlockResponseQuery};
use sui_sdk::types::digests::TransactionDigest;
use sui_sdk::{types::query::TransactionFilter, SuiClient, SuiClientBuilder};
pub struct ChainCli {
    cli: SuiClient,
}

impl ChainCli {
    async fn new(url: &str) -> Self {
        let sui = SuiClientBuilder::default().build(url).await.unwrap();
        Self { cli: sui }
    }

    async fn get_tx_version(&self) -> u64 {
        self.cli
            .read_api()
            .get_total_transaction_blocks()
            .await
            .unwrap()
    }

    async fn get_tx_digest(&self, checkpoint: u64) -> Vec<SuiTransactionBlockResponse> {
        let checkpoint_seq_query = SuiTransactionBlockResponseQuery::new(
            Some(TransactionFilter::Checkpoint(checkpoint)),
            Some(SuiTransactionBlockResponseOptions::new().with_input().with_events()),
        );

        let txs = self
            .cli
            .read_api()
            .get_transactions_stream(checkpoint_seq_query, None, true)
            .collect::<Vec<_>>()
            .await;
        txs
    }

    async fn get_tx_detail(&self, digest: TransactionDigest) -> SuiTransactionBlockResponse {
        let response: SuiTransactionBlockResponse = self.cli
            .read_api()
            .get_transaction_with_options(digest, SuiTransactionBlockResponseOptions::new().with_input().with_events())
            .await
            .unwrap();
        response
    }
}

#[cfg(test)]
mod tests {
    use crate::sui::chain::ChainCli;

    #[test]
    fn test_get_total_tx_number() {
        let rt = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap();

        let cli = rt.block_on(ChainCli::new("https://fullnode.devnet.sui.io:443"));
        let num = rt.block_on(cli.get_tx_version());
        println!("{}", num);
        assert!(num > 50799, "split?");
    }

    #[test]
    fn test_get_tx_digest() {
        let rt = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap();

        let cli = rt.block_on(ChainCli::new("https://fullnode.testnet.sui.io:443"));
        let txs: Vec<sui_json_rpc_types::SuiTransactionBlockResponse> = rt.block_on(cli.get_tx_digest(390753u64));
        println!("{:?}", txs);
        assert!(!txs.is_empty(), "not tx in checkpoint");
        // for tx in txs {
        //     let tx_detail = rt.block_on(cli.get_tx_detail(tx.digest));
        //     println!("{:?}", tx_detail);
        // }
    }
}
