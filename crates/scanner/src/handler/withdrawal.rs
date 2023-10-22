use std::str::FromStr;

use bigdecimal::BigDecimal;
use chrono::Utc;
use entities::withdrawals::Model;
use ethers::types::{Withdrawal, H256};

pub fn withdrawals_process(
    block_hash: Option<H256>,
    withdrawals: Option<Vec<Withdrawal>>,
) -> Vec<Model> {
    let mut res = vec![];
    if let Some(withdrawals) = withdrawals {
        for item in withdrawals.iter() {
            let model = Model {
                index: item.index.as_u32() as i32,
                validator_index: item.validator_index.as_u32() as i32,
                amount: BigDecimal::from_str(item.amount.to_string().as_str()).unwrap(),
                inserted_at: Utc::now().naive_utc(),
                updated_at: Utc::now().naive_utc(),
                address_hash: item.address.as_bytes().to_vec(),
                block_hash: match block_hash {
                    Some(hash) => hash.as_bytes().to_vec(),
                    None => vec![],
                },
            };
            res.push(model);
        }
    }

    res
}
