use ethers::types::{Log, TransactionReceipt, H160};

use crate::common::consts;

pub struct MintTransfer {
    pub from: Vec<u8>,
    pub to: Vec<u8>,
    pub block_number: u64,
}

pub fn parse_mint_transfer(receipts: &[TransactionReceipt]) -> Vec<MintTransfer> {
    let mut mint_transfers = vec![];
    for receipt in receipts.iter() {
        let mut tmp = parse(&receipt.logs);
        mint_transfers.append(&mut tmp);
    }

    mint_transfers
}

fn parse(logs: &[Log]) -> Vec<MintTransfer> {
    let mut mint_transfers: Vec<MintTransfer> = Vec::new();

    for log in logs.iter() {
        if log.topics[0].as_bytes() == consts::BRIDGE_HASH.as_bytes() {
            if let Some(params) = parse_params(&log) {
                mint_transfers.push(params);
            }
        }
    }

    mint_transfers
}

fn parse_params(log: &Log) -> Option<MintTransfer> {
    if let (second_topic, third_topic, Some(block_number)) =
        (log.topics[1], log.topics[2], log.block_number)
    {
        let to = H160::from(second_topic).as_bytes().to_vec();
        let from = H160::from(third_topic).as_bytes().to_vec();
        return Some(MintTransfer {
            block_number: block_number.as_u64(),
            from,
            to,
        });
    }

    None
}
