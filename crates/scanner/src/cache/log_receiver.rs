use repo::dal::log_receiver_contract::Query as ContractQuery;
use sea_orm::DbConn;
use std::collections::HashMap;

#[derive(Clone)]
pub struct LogReceiverContract {
    pub chain_name: String,
    pub chain_id: u32,
    pub address: String,
    pub event_sign: String,
}

impl LogReceiverContract {
    pub fn cache_key(&self) -> String {
        format!(
            "{}-{}-{}-{}",
            self.chain_name, self.chain_id, self.address, self.event_sign
        )
    }
}

pub struct ContractAddrCache {
    contact_map: HashMap<String, LogReceiverContract>,
}

impl Default for ContractAddrCache {
    fn default() -> Self {
        Self::new()
    }
}

impl ContractAddrCache {
    pub fn new() -> Self {
        Self {
            contact_map: HashMap::new(),
        }
    }

    pub fn insert(&mut self, k: String, v: LogReceiverContract) {
        self.contact_map.insert(k, v);
    }

    pub fn find(&self, k: String) -> LogReceiverContract {
        let res = self.contact_map.get(&k).unwrap();
        res.clone()
    }

    pub fn exist(&self, k: String) -> bool {
        self.contact_map.contains_key(&k)
    }
}

#[allow(dead_code)]
async fn update_contract_cache(conn: &DbConn) -> ContractAddrCache {
    let mut contract_addr_cache: ContractAddrCache = ContractAddrCache::new();
    let (contracts, _) = ContractQuery::find_scanner_contract_in_page(conn, 1, 100)
        .await
        .unwrap();
    for v in contracts {
        let data = LogReceiverContract {
            chain_name: v.chain_name,
            chain_id: v.chain_id,
            address: v.address,
            event_sign: v.event_sign,
        };
        contract_addr_cache.insert(data.cache_key(), data);
    }

    contract_addr_cache
}
