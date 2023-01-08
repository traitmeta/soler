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
    fn handle_log(&self, log: web3::types::Log) -> Result<(), Box<dyn Error>>;
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
    opr_timestamp: i32,
}

pub struct SBT {
    contract_map: HashMap<String, bool>,
    insert_queue: Vec<SBTInfo>,
    update_queue: Vec<SBTInfo>,
    delete_queue: Vec<SBTInfo>,
}

impl EventHandler for SBT {
    fn handle(&self) -> Result<(), Box<dyn Error>> {
        Ok(())
    }

    fn handle_log(&self, log: web3::types::Log) -> Result<(), Box<dyn Error>> {
        let contract_addr = log.topics.get(0);

        let value = match self.contract_map.get(&log.address.to_string()){
            Some(exist) =>  true,
            None =>  return Ok(())
        };
        
        Ok(())
    }
}
