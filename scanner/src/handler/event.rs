use chrono::Utc;
use entities::logs::Model as LogModel;
use ethers::types::TransactionReceipt;

pub fn handle_block_event(receipts: &Vec<TransactionReceipt>) -> Vec<LogModel> {
    let mut events = Vec::new();
    for receipt in receipts.iter() {
        for log in receipt.logs.iter() {
            // tracing::debug!("handle_block_event, log: {:?}", log);
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
                block_number: match log.block_number {
                    Some(number) => Some(number.as_u64() as i32),
                    None => None,
                },
                inserted_at: Utc::now().naive_utc(),
                updated_at: Utc::now().naive_utc(),
            };

            for (i, topic) in log.topics.iter().enumerate() {
                match i {
                    0 => event.first_topic = Some(topic.to_string()),
                    1 => event.second_topic = Some(topic.to_string()),
                    2 => event.third_topic = Some(topic.to_string()),
                    3 => event.fourth_topic = Some(topic.to_string()),
                    _ => (),
                }
            }
            events.push(event);
        }
    }

    events
}
