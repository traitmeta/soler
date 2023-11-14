use anyhow::Result;
use bigdecimal::BigDecimal;
use chrono::Utc;
use ethers::types::U256;
use std::{collections::HashMap, str::FromStr, time::Duration};
use tokio::time;
use tracing::error;

use entities::address_current_token_balances::Model as CurrentTokenBalanceModel;
use entities::address_token_balances::Model as TokenBalanceModel;

use crate::{
    common::consts,
    contracts::balance_reader::{BalanceReader, TokenBalanceRequest},
};

// First: get data from db and call contract get balance. this will make all data do it.
// Second: use channel when address token balance model handler and sender it to channel. then balance fether received message and get balance form call contract
pub async fn fetch_token_balances_from_blockchain(
    balance_reader: BalanceReader,
    token_balances: Vec<TokenBalanceModel>,
) -> Result<HashMap<String, Vec<TokenBalanceModel>>> {
    println!("fetching token balances, count: {}", token_balances.len());

    let mut fetched_token_balances = vec![];
    let mut failed_token_balances = vec![];
    for token_balance in token_balances.iter() {
        let req = TokenBalanceRequest {
            token_contract_address_hash: format!(
                "0x{}",
                hex::encode(token_balance.token_contract_address_hash.clone())
            ),
            address_hash: format!("0x{}", hex::encode(token_balance.address_hash.clone())),
            block_number: Some(token_balance.block_number as u64),
            token_id: token_balance
                .token_id
                .as_ref()
                .map(|token_id| U256::from(token_id.to_string().as_bytes())),
            token_type: token_balance
                .token_type
                .as_ref()
                .map_or(consts::TokenKind::None, |t| match t.as_str() {
                    consts::ERC20 => consts::TokenKind::ERC20,
                    consts::ERC721 => consts::TokenKind::ERC721,
                    consts::ERC1155 => consts::TokenKind::ERC1155,
                    _ => consts::TokenKind::None,
                }),
        };
        let result = balance_reader.token_balance_call_contract(req).await;
        let mut item = token_balance.clone();
        item.value_fetched_at = Some(Utc::now().naive_utc());
        match result {
            Ok(balance) => {
                item.value = Some(BigDecimal::from_str(balance.to_string().as_str()).unwrap());
                fetched_token_balances.push(item);
            }
            Err(_) => {
                failed_token_balances.push(item);
            }
        }
    }

    let mut token_balances_map = HashMap::new();
    for token_balance in fetched_token_balances {
        let key = get_token_balance_key(&token_balance);
        token_balances_map
            .entry(key)
            .or_insert_with(Vec::new)
            .push(token_balance.clone());
    }

    Ok(token_balances_map)
}

fn get_token_balance_key(token_balance: &TokenBalanceModel) -> String {
    format!(
        "0x{}_{}_0x{}",
        hex::encode(token_balance.token_contract_address_hash.clone()),
        token_balance
            .token_id
            .as_ref()
            .unwrap_or(&BigDecimal::from(0)),
        hex::encode(token_balance.address_hash.clone()),
    )
}

pub async fn fetch_current_token_balance(
    balance_reader: BalanceReader,
    token_balance: &mut CurrentTokenBalanceModel,
) -> Result<()> {
    let backoff = 1;

    loop {
        let req = TokenBalanceRequest {
            token_contract_address_hash: format!(
                "0x{}",
                hex::encode(token_balance.token_contract_address_hash.clone())
            ),
            address_hash: format!("0x{}", hex::encode(token_balance.address_hash.clone())),
            block_number: Some(token_balance.block_number as u64),
            token_id: token_balance
                .token_id
                .as_ref()
                .map(|token_id| U256::from(token_id.to_string().as_bytes())),
            token_type: token_balance
                .token_type
                .as_ref()
                .map_or(consts::TokenKind::None, |t| match t.as_str() {
                    consts::ERC20 => consts::TokenKind::ERC20,
                    consts::ERC721 => consts::TokenKind::ERC721,
                    consts::ERC1155 => consts::TokenKind::ERC1155,
                    _ => consts::TokenKind::None,
                }),
        };

        let result = balance_reader.token_balance_call_contract(req).await;
        token_balance.value_fetched_at = Some(Utc::now().naive_utc());
        token_balance.inserted_at = Utc::now().naive_utc();
        token_balance.updated_at = Utc::now().naive_utc();
        token_balance.old_value = token_balance.value.clone();
        match result {
            Ok(balance) => {
                token_balance.value =
                    Some(BigDecimal::from_str(balance.to_string().as_str()).unwrap());
                return Ok(());
            }
            Err(e) => {
                if backoff > 64 {
                    error!("failed to accept socket after retry: {}", e);
                    return Err(e);
                } else {
                    error!("failed to accept socket: {}", e);
                }
            }
        }

        time::sleep(Duration::from_secs(backoff)).await;

        let _ = backoff << 2;
    }
}
