/*
 SBT handler's main function is hanlder event and change logic status or notify upstream
function:
1. SBT lifetime, mint or burn
2. record the history for all event
3. maintain the status of lifetime.

minting --> minted --> burning --> burned
^    ｜                     |
|    ｜                     |
-- mint_fail           burn_fail
*/

use std::{
    collections::HashMap,
    error::{self, Error},
};

pub trait EventHandler {
    fn handle(&self) -> Result<(), Box<dyn Error>>;
    fn handle_log(&self, log: web3::types::Log, block_timestamp: i64)
        -> Result<(), Box<dyn Error>>;
    fn handle_insert(
        &self,
        log: web3::types::Log,
        block_timestamp: i64,
    ) -> Result<(), Box<dyn Error>>;
}

#[repr(u8)]
enum OprType {
    Unkonw = 0,
    Mint = 1,
    Renewal = 2,
    Burn = 3,
}

struct SBTInfo {
    chain_id: u64,
    contract: String,
    sbt_id: String,
    token_id: String,
    status: u8,
    lifetime: SBTLifetime,
}

struct SBTLifetime {
    sbt_info_id: u64,
    opr_type: u8,
    opr_timestamp: i64,
}

pub struct SBT {
    contract_map: HashMap<String, bool>,
    insert_queue: Vec<SBTInfo>,
    update_queue: Vec<SBTInfo>,
    delete_queue: Vec<SBTInfo>,
}

impl SBT {
    fn new() -> Self {
        SBT {
            contract_map: HashMap::new(),
            insert_queue: vec![],
            update_queue: vec![],
            delete_queue: vec![],
        }
    }

    fn add_contract_to_map(&mut self, contract: String) {
        self.contract_map.insert(contract, true);
    }

    fn get_opr_type_from_address(&self, addr: &str) -> OprType {
        match addr {
            "0x12345" => return OprType::Mint,
            "0x22345" => return OprType::Renewal,
            "0x32345" => return OprType::Burn,
            _ => OprType::Unkonw,
        }
    }
}

impl EventHandler for SBT {
    fn handle(&self) -> Result<(), Box<dyn Error>> {
        Ok(())
    }

    fn handle_log(
        &self,
        log: web3::types::Log,
        block_timestamp: i64,
    ) -> Result<(), Box<dyn Error>> {
        let contract_addr = log.topics.get(0);

        let value = match self.contract_map.get(&log.address.to_string()) {
            Some(exist) => true,
            None => return Ok(()),
        };

        Ok(())
    }

    fn handle_insert(
        &self,
        log: web3::types::Log,
        block_timestamp: i64,
    ) -> Result<(), Box<dyn Error>> {
        let topic  = log.topics.get(0).unwrap();
        let insert_info = SBTLifetime {
            sbt_info_id: todo!(),
            opr_type: self.get_opr_type_from_address(topic.to_string().as_str()) as u8,
            opr_timestamp: block_timestamp,
        };

        Ok(())
    }
}
