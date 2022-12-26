use std::collections::HashMap;

#[derive(Clone)]
pub struct ScannerContract {
    pub chain_name: String,
    pub chain_id: u32,
    pub address: String,
    pub event_sign: String,
}

impl ScannerContract {
    pub fn cache_key(&self) -> String {
        format!(
            "{}-{}-{}-{}",
            self.chain_name, self.chain_id, self.address, self.event_sign
        )
    }
}

pub struct ContractAddrCache {
    contact_map: HashMap<String, ScannerContract>,
}

impl ContractAddrCache {
    pub fn new() -> Self {
        Self {
            contact_map: HashMap::new(),
        }
    }

    pub fn insert(&mut self, k: String, v: ScannerContract) {
        self.contact_map.insert(k, v);
    }

    pub fn find(&self, k: String) -> ScannerContract {
        let res = self.contact_map.get(&k).unwrap();
        res.clone()
    }

    pub fn exist(&self, k: String) -> bool {
        self.contact_map.contains_key(&k)
    }
}
