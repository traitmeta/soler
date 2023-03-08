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

use std::{collections::HashMap, error::Error};

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

#[derive(Copy, Clone)]
enum OprType {
    Unkonw,
    Mint,
    Burn,
}

struct SBTEvent {
    chain_id: u64,
    contract: String,
    block_timestamp: i64,
    event_topic_addr: String,
}

struct SBTMintEvent {
    who: String,
    token_id: String,
    sbt_id: String,
    base: SBTEvent,
}

struct SBTBurnEvent {
    who: String,
    token_id: String,
    base: SBTEvent,
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
    contract_type_map: HashMap<String, OprType>,
    contract_map: HashMap<String, bool>,
    insert_queue: Vec<SBTInfo>,
    update_queue: Vec<SBTInfo>,
    delete_queue: Vec<SBTInfo>,
}

impl SBT {
    fn new() -> Self {
        SBT {
            contract_type_map: HashMap::new(),
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
        match self.contract_type_map.get(addr) {
            Some(opr_type) => *opr_type,
            None => OprType::Unkonw,
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
        let topic = log.topics.get(0).unwrap();

        // TODO 从LOG获取改为从自定义Model获取
        let sbt_info = SBTInfo {
            chain_id: todo!(),
            contract: todo!(),
            sbt_id: todo!(),
            token_id: todo!(),
            status: todo!(),
            lifetime: todo!(),
        };
        let insert_info = SBTLifetime {
            sbt_info_id: todo!(),
            opr_type: self.get_opr_type_from_address(topic.to_string().as_str()) as u8,
            opr_timestamp: block_timestamp,
        };

        Ok(())
    }
}
