use serde::{Deserialize, Serialize};
use std::{fmt::{self, Display, Formatter}, sync::mpsc::Sender};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BroadcastType {
    OnDamend,
    None,
}

impl Display for BroadcastType {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(
          f,
          "{}",
          match self {
            Self::OnDamend => "OnDamend",
            Self::None => "",
          }
        )
      }
}

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

    pub async fn broadcast(&self, data: Vec<(String, String)>, broadcast_type: BroadcastType) {
        for (event_type, event_data) in data {
            if self.allowed_events.contains(&event_type) {
                self.send_data(&event_type, broadcast_type.clone(), &event_data);
            }
        }
    }

    pub fn broadcast_single(&self, event_type: String) {
        if self.allowed_events.contains(&event_type) {
            self.send_data(&event_type, BroadcastType::None, "");
        }
    }

    fn send_data(&self, event_type: &str, broadcast_type: BroadcastType, event_data: &str) {
        let message = format!("{} {} {}", event_type, broadcast_type, event_data);
        self.sender.send(message).unwrap();
    }
}
