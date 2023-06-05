use sui_json_rpc_types::SuiEvent;

use crate::{model::events::Event};

use super::chain::ChainCli;

pub struct SuiEventListener {
    sui_client: ChainCli,
}

impl SuiEventListener {
    pub async fn new(url: &str) -> Self {
        SuiEventListener {
            sui_client: ChainCli::new(url).await,
        }
    }

    pub async fn listener_event(&self, checkpoint: u64) -> Vec<Event> {
        let mut sui_events: Vec<Event> = vec![];
        let txs = self.sui_client.get_tx_stream(checkpoint).await;
        assert!(!txs.is_empty(), "not tx in checkpoint");
        for tx in txs {
            if let Some(events) = tx.events {
                if events.data.is_empty() {
                    continue;
                }

                let mut tx_events = self.build_event_model(
                    tx.checkpoint.unwrap_or_default(),
                    tx.timestamp_ms.unwrap_or_default(),
                    tx.digest.to_string(),
                    events.data,
                );
                sui_events.append(&mut tx_events);
            }
        }
        sui_events
    }

    fn build_event_model(
        &self,
        height: u64,
        ts_ms: u64,
        tx_id: String,
        events_data: Vec<SuiEvent>,
    ) -> Vec<Event> {
        let mut sui_events: Vec<Event> = vec![];
        for event in events_data.iter() {
            let event_model = Event {
                height,
                timestamp_ms: ts_ms,
                tx_id: tx_id.to_owned(),
                contract_adress: event.package_id.to_string(),
                event_name: event.type_.name.clone().into_string(),
                event_content: event.parsed_json.to_string(),
            };
            sui_events.push(event_model);
        }

        sui_events
    }
}

#[cfg(test)]
mod event_tests {
    use crate::sui::event::SuiEventListener;

    #[test]
    fn test_get_tx_digest() {
        let rt = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap();

        let cli = rt.block_on(SuiEventListener::new("https://fullnode.testnet.sui.io:443"));
        let txs = rt.block_on(cli.listener_event(390753u64));
        // println!("{:?}", txs);
        assert!(!txs.is_empty(), "not tx in checkpoint");
    }
}
