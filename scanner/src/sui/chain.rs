use std::str::FromStr;

use futures::StreamExt;
use move_core_types::identifier::Identifier;
use sui_json_rpc_types::SuiEvent;
use sui_json_rpc_types::{
    EventFilter, SuiTransactionBlockResponse, SuiTransactionBlockResponseOptions,
    SuiTransactionBlockResponseQuery, TransactionFilter,
};
use sui_sdk::types::base_types::ObjectID;
use sui_sdk::types::digests::TransactionDigest;
use sui_sdk::types::event::EventID;
use sui_sdk::{SuiClient, SuiClientBuilder};

pub struct ChainCli {
    cli: SuiClient,
}

impl ChainCli {
    pub async fn new(url: &str) -> Self {
        let sui = SuiClientBuilder::default().build(url).await.unwrap();
        Self { cli: sui }
    }

    pub async fn get_tx_version(&self) -> u64 {
        self.cli
            .read_api()
            .get_total_transaction_blocks()
            .await
            .unwrap()
    }

    pub async fn get_tx_stream(&self, checkpoint: u64) -> Vec<SuiTransactionBlockResponse> {
        let checkpoint_seq_query = SuiTransactionBlockResponseQuery::new(
            Some(TransactionFilter::Checkpoint(checkpoint)),
            Some(
                SuiTransactionBlockResponseOptions::new()
                    .with_input()
                    .with_events(),
            ),
        );

        self.cli
            .read_api()
            .get_transactions_stream(checkpoint_seq_query, None, true)
            .collect::<Vec<_>>()
            .await
    }

    // NOT WORK
    pub async fn get_events_stream(&self, package_id: &str) -> Vec<SuiEvent> {
        let package_id_obj = ObjectID::from_hex_literal(package_id).unwrap();
        let package_id_filter = EventFilter::Package(package_id_obj);
        let move_module_filter = EventFilter::MoveModule {
            package: package_id_obj,
            module: Identifier::new("NFTMinted").unwrap(),
        };
        let event_filters = EventFilter::All(vec![package_id_filter, move_module_filter]);

        self.cli
            .event_api()
            .get_events_stream(event_filters, None, true)
            .collect::<Vec<_>>()
            .await
    }

    pub async fn get_events_by_packege_and_module(&self, package_id: &str, module_name: &str) -> Vec<SuiEvent> {
        let package_id_obj = ObjectID::from_hex_literal(package_id).unwrap();
        // full node not support package filter
        // let package_id_filter = EventFilter::Package(package_id_obj);
        let move_module_filter = EventFilter::MoveModule {
            package: package_id_obj,
            module: Identifier::new(module_name).unwrap(),
        };

        let page = self
            .cli
            .event_api()
            .query_events(move_module_filter, None, None, true)
            .await
            .ok()
            .unwrap();

        page.data
    }

    pub async fn get_events(&self, package_id: &str, module_name: &str) -> Vec<SuiEvent> {
        let package_id_obj = ObjectID::from_hex_literal(package_id).unwrap();
        // full node not support package filter
        // let package_id_filter = EventFilter::Package(package_id_obj);
        let move_module_filter = EventFilter::MoveModule {
            package: package_id_obj,
            module: Identifier::new(module_name).unwrap(),
        };

        let event_id: EventID = EventID {
            tx_digest: TransactionDigest::from_str("6zHXRuGgxKrVhmSmChwHNFh4eRb9rRBbhcDi6wjfHovj")
                .unwrap(),
            event_seq: 3249988,
        };

        // let event_filters = EventFilter::All(vec![package_id_filter, move_module_filter]);
        let page = self
            .cli
            .event_api()
            .query_events(move_module_filter, Some(event_id), Some(10), true)
            .await
            .ok();

        match page {
            None => vec![],
            Some(data) => data.data,
        }
    }

    pub async fn get_tx_detail(&self, digest: TransactionDigest) -> SuiTransactionBlockResponse {
        let response: SuiTransactionBlockResponse = self
            .cli
            .read_api()
            .get_transaction_with_options(
                digest,
                SuiTransactionBlockResponseOptions::new()
                    .with_input()
                    .with_events(),
            )
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
        let txs: Vec<sui_json_rpc_types::SuiTransactionBlockResponse> =
            rt.block_on(cli.get_tx_stream(390753u64));
        // println!("{:?}", txs);
        assert!(!txs.is_empty(), "not tx in checkpoint");
        for tx in txs {
            if let Some(events) = tx.events {
                if events.data.is_empty() {
                    continue;
                }
                println!("{:?}", events);
            }
        }
    }

    #[test]
    fn test_get_events() {
        let rt = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap();

        let cli = rt.block_on(ChainCli::new("https://fullnode.testnet.sui.io:443"));
        let events = rt.block_on(cli.get_events(
            "0x5ea6aafe995ce6506f07335a40942024106a57f6311cb341239abf2c3ac7b82f",
            "nft",
        ));
        // 只有两个EVENT， NFT MINT 没有EVENT
        assert!(events.len()>0, "events in giving package id");
    }

    #[test]
    fn test_get_events_v1() {
        let rt = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap();

        let cli = rt.block_on(ChainCli::new("https://fullnode.testnet.sui.io:443"));
        let events = rt.block_on(cli.get_events_by_packege_and_module(
            "0x4247e72df58552701456293027e75237fe85a214cd050b6e0358dc5047a3fb17",
            "aggregator_save_result_action",
        ));

        // 有很多EVENT
        assert!(events.len()>0, "events in giving package id");
    }
}
