pub mod block_number;
pub mod log_receiver;
use std::sync::mpsc::{Sender, Receiver};

pub struct Publisher {
    allowed_events: Vec<String>,
    sender: Sender<String>,
}

impl Publisher {
    pub fn new(sender: Sender<String>) -> Self {
        let allowed_events = vec![
            "addresses".to_string(),
            "address_coin_balances".to_string(),
            "address_token_balances".to_string(),
            "address_current_token_balances".to_string(),
            "blocks".to_string(),
            "block_rewards".to_string(),
            "internal_transactions".to_string(),
            "last_block_number".to_string(),
            "token_transfers".to_string(),
            "transactions".to_string(),
            "contract_verification_result".to_string(),
            "token_total_supply".to_string(),
            "changed_bytecode".to_string(),
            "smart_contract_was_verified".to_string(),
        ];

        Publisher {
            allowed_events,
            sender,
        }
    }

    pub fn broadcast(&self, data: Vec<(String, String)>, broadcast_type: bool) {
        for (event_type, event_data) in data {
            if self.allowed_events.contains(&event_type) {
                self.send_data(&event_type, broadcast_type, &event_data);
            }
        }
    }

    pub fn broadcast_single(&self, event_type: String) {
        if self.allowed_events.contains(&event_type) {
            self.send_data(&event_type, false, "");
        }
    }

    fn send_data(&self, event_type: &str, broadcast_type: bool, event_data: &str) {
        let message = format!("{} {} {}", event_type, broadcast_type, event_data);
        self.sender.send(message).unwrap();
    }
}
```