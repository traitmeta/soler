use chrono::Utc;
use common::chain_ident;
use entities::logs::Model as LogModel;
use ethers::types::TransactionReceipt;

pub fn handle_block_event(receipts: &[TransactionReceipt]) -> Vec<LogModel> {
    let mut events = Vec::new();
    for receipt in receipts.iter() {
        for log in receipt.logs.iter() {
            let mut event = LogModel {
                data: log.data.to_vec(),
                index: match log.log_index {
                    Some(index) => index.as_u64() as i32,
                    None => 0,
                },
                r#type: log.log_type.clone(),
                first_topic: None,
                second_topic: None,
                third_topic: None,
                fourth_topic: None,
                address_hash: Some(log.address.as_bytes().to_vec()),
                transaction_hash: match log.transaction_hash {
                    Some(hash) => hash.as_bytes().to_vec(),
                    None => vec![],
                },
                block_hash: match log.block_hash {
                    Some(hash) => hash.as_bytes().to_vec(),
                    None => vec![],
                },
                block_number: log.block_number.map(|number| number.as_u64() as i32),
                inserted_at: Utc::now().naive_utc(),
                updated_at: Utc::now().naive_utc(),
            };

            for (i, topic) in log.topics.iter().enumerate() {
                let tp = Some(chain_ident!(topic.as_bytes()));
                match i {
                    0 => event.first_topic = tp,
                    1 => event.second_topic = tp,
                    2 => event.third_topic = tp,
                    3 => event.fourth_topic = tp,
                    _ => (),
                }
            }
            events.push(event);
        }
    }

    events
}
