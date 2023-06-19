use std::str::FromStr;

use futures::StreamExt;
use move_core_types::identifier::Identifier;
use serde::de::value::Error;
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

    // NOT WORK
    pub async fn get_events(&self, package_id: &str) -> Vec<SuiEvent> {
        let package_id_obj = ObjectID::from_hex_literal(package_id).unwrap();
        // full node not support package filter
        // let package_id_filter = EventFilter::Package(package_id_obj);
        let move_module_filter = EventFilter::MoveModule {
            package: package_id_obj,
            module: Identifier::new("NFTMinted").unwrap(),
        };

        let event_id: EventID = EventID {
            tx_digest: TransactionDigest::from_str("8NbvGRuCaFmHU9z7hGEtiWkEwkVG8q7jtZAYneA3Vh15")
                .unwrap(),
            event_seq: 2134770,
        };

        // let event_filters = EventFilter::All(vec![package_id_filter, move_module_filter]);
        let page = self
            .cli
            .event_api()
            .query_events(move_module_filter, Some(event_id), Some(100), true)
            .await
            .ok();

        match page {
            None => vec![],
            Some(data) => data.data
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
        let events: Vec<sui_json_rpc_types::SuiEvent> = rt.block_on(
            cli.get_events("0xfcb0c2f067d41f0d1da637fe929cfbb5435bf629a059a259c75e60c1ee550f0a"),
        );
        assert!(events.is_empty(), "events in giving package id");
    }
}
