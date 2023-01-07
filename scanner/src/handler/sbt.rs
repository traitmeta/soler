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

pub trait EventHandler {
    fn handle(&self) -> Result<(), error>;
}

struct SBTInfo {
    chain_id: u64,
    contract: string,
    sbt_id: string,
    token_id: string,
    status: u8,
    lifetime: SBTLifetime,
}

struct SBTLifetime {
    sbt_info_id: u64,
    opr_type: u8,
    opr_timestamp: i32,
}

pub struct SBT {
    insert_queue: Vec<SBTInfo>,
    update_queue: Vec<SBTInfo>,
    delete_queue: Vec<SBTInfo>,
}

impl EventHandler for SBT {
    fn handle(&self) -> Result<(), error> {
        OK(())
    }
}
