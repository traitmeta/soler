use anyhow::Result;
use bigdecimal::BigDecimal;
use chrono::Utc;
use ethers::types::U256;
use std::{str::FromStr, time::Duration};
use tokio::time;
use tracing::error;

use entities::address_current_token_balances::Model as CurrentTokenBalanceModel;
use entities::address_token_balances::Model as TokenBalanceModel;

use crate::contracts::balance_reader::{BalanceReader, TokenBalanceRequest};
use common::{chain_ident, consts};

// should be get balance with two ways, first is async, get data which have field block_number from db and call contract;
// second is sync, when fetching and parsed data from block, send it to channel.
pub async fn fetch_token_balances_from_blockchain(
    balance_reader: &BalanceReader,
    token_balances: Vec<TokenBalanceModel>,
) -> Result<Vec<TokenBalanceModel>> {
    let mut fetched_token_balances = vec![];
    // let mut failed_token_balances = vec![];
    for token_balance in token_balances.iter() {
        let token_hash = chain_ident!(token_balance.token_contract_address_hash.clone());
        let address_hash = chain_ident!(token_balance.address_hash.clone());
        tracing::info!(
            "update address token => token balance: {:?}, address_hash :{:?}",
            token_hash.clone(),
            address_hash.clone()
        );
        let req = TokenBalanceRequest {
            token_hash,
            address_hash,
            block_number: Some(token_balance.block_number as u64),
            token_id: token_balance
                .token_id
                .as_ref()
                .map(|token_id| token_id.to_string()),
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
                // failed_token_balances.push(item);
            }
        }
    }

    // let mut token_balances_map = HashMap::new();
    // for token_balance in fetched_token_balances {
    //     let key = get_token_balance_key(&token_balance);
    //     token_balances_map
    //         .entry(key)
    //         .or_insert_with(Vec::new)
    //         .push(token_balance.clone());
    // }

    Ok(fetched_token_balances)
}

fn _get_token_balance_key(token_balance: &TokenBalanceModel) -> String {
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

// This function fetches the current token balance asynchronously.
// It takes a `BalanceReader` object, a mutable reference to a `CurrentTokenBalanceModel` object, and a `max_block_number` of type `u64` as input parameters.
// It returns a `Result` type, which is `()` meaning there is no value to return in case of success, and an error in case of failure.
pub async fn fetch_current_token_balance(
    balance_reader: BalanceReader,
    token_balance: &mut CurrentTokenBalanceModel,
    max_block_number: u64,
) -> Result<()> {
    let backoff = 1;

    if max_block_number - token_balance.block_number as u64 <= 1 {
        return Ok(());
    }

    loop {
        let req = TokenBalanceRequest {
            token_hash: format!(
                "0x{}",
                hex::encode(token_balance.token_contract_address_hash.clone())
            ),
            address_hash: chain_ident!(token_balance.address_hash.clone()),
            block_number: Some(token_balance.block_number as u64),
            token_id: token_balance
                .token_id
                .as_ref()
                .map(|token_id| token_id.to_string()),
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
