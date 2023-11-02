use bigdecimal::BigDecimal;
use chrono::Utc;
use ethers::types::U256;
use std::collections::HashMap;

use entities::address_token_balances::Model as TokenBalanceModel;

use crate::{
    common::consts,
    contracts::balance_reader::{BalanceReader, TokenBalanceRequest},
};

// First: get data from db and call contract get balance. this will make all data do it.
// Second: use channel when address token balance model handler and sender it to channel. then balance fether received message and get balance form call contract
#[derive(Debug)]
struct TokenBalance {
    address_hash: String,
    token_contract_address_hash: String,
    token_id: Option<BigDecimal>,
    block_number: u64,
    value: Option<u64>,
    value_fetched_at: Option<chrono::DateTime<Utc>>,
    error: Option<String>,
}

impl TokenBalance {
    fn set_value(&mut self, balance: Result<u64, String>) {
        match balance {
            Ok(value) => {
                self.value = Some(value);
                self.value_fetched_at = Some(Utc::now());
                self.error = None;
            }
            Err(error) => {
                self.value = None;
                self.value_fetched_at = None;
                self.error = Some(error);
            }
        }
    }
}

fn fetch_token_balances_from_blockchain(
    token_balances: Vec<TokenBalanceModel>,
) -> Result<HashMap<String, Vec<TokenBalanceModel>>, String> {
    println!("fetching token balances, count: {}", token_balances.len());

    let (regular_token_balances, erc1155_token_balances): (
        Vec<TokenBalanceModel>,
        Vec<TokenBalanceModel>,
    ) = token_balances.into_iter().partition(|request| {
        if let Some(token_type) = &request.token_type {
            token_type != "ERC-1155"
        } else {
            true
        }
    });

    let requested_regular_token_balances: Vec<TokenBalanceModel> = regular_token_balances
        .into_iter()
        .map(|token_balance| {
            let req = TokenBalanceRequest {
                token_contract_address_hash: format!(
                    "0x{}",
                    hex::encode(token_balance.token_contract_address_hash)
                ),
                address_hash: format!("0x{}", hex::encode(token_balance.address_hash)),
                block_number: token_balance.block_number,
                token_id: token_balance
                    .token_id
                    .map(|token_id| U256::from(token_id.to_string().as_bytes())),
                token_type: token_balance
                    .token_type
                    .map_or(consts::TokenKind::None, |t| match t.as_str() {
                        consts::ERC20 => consts::TokenKind::ERC20,
                        consts::ERC721 => consts::TokenKind::ERC721,
                        consts::ERC1155 => consts::TokenKind::ERC1155,
                        _ => consts::TokenKind::None,
                    }),
            };
            let result = BalanceReader::token_balance_call_contract(req);
            set_token_balance_value(result, token_balance)
        })
        .collect();

    let requested_erc1155_token_balances: Vec<TokenBalanceModel> = erc1155_token_balances
        .into_iter()
        .map(|token_balance| {
            let result = BalanceReader::get_balances_of(token_balance);
            set_token_balance_value(result, token_balance)
        })
        .collect();

    let mut requested_token_balances = requested_regular_token_balances;
    requested_token_balances.extend(requested_erc1155_token_balances);

    let fetched_token_balances: Vec<TokenBalance> = requested_token_balances
        .iter()
        .filter(|token_balance| token_balance.error.is_none())
        .cloned()
        .collect();

    let failed_token_balances: Vec<TokenBalance> = requested_token_balances
        .into_iter()
        .filter(|token_balance| token_balance.error.is_some())
        .collect();

    let mut token_balances_map: HashMap<String, Vec<TokenBalance>> = HashMap::new();
    for token_balance in fetched_token_balances {
        let key = format!(
            "{}_{}_{}",
            token_balance.token_contract_address_hash,
            token_balance.token_id.unwrap_or(0),
            token_balance.address_hash
        );
        token_balances_map
            .entry(key)
            .or_insert_with(Vec::new)
            .push(token_balance);
    }

    Ok(token_balances_map)
}

fn set_token_balance_value(
    balance: Result<u64, String>,
    mut token_balance: TokenBalance,
) -> TokenBalance {
    match balance {
        Ok(value) => {
            token_balance.value = Some(value);
            token_balance.value_fetched_at = Some(Utc::now());
            token_balance.error = None;
        }
        Err(error) => {
            token_balance.value = None;
            token_balance.value_fetched_at = None;
            token_balance.error = Some(error);
        }
    }
    token_balance
}

fn handle_killed_tasks(
    requested_token_balances: &[TokenBalance],
    token_balances: &[TokenBalance],
) -> Vec<TokenBalance> {
    token_balances
        .iter()
        .filter(|token_balance| !present(requested_token_balances, token_balance))
        .map(|token_balance| {
            let mut new_token_balance = token_balance.clone();
            new_token_balance.value = None;
            new_token_balance.value_fetched_at = None;
            new_token_balance.error = Some("timeout".to_string());
            new_token_balance
        })
        .collect()
}

fn present(list: &[TokenBalance], token_balance: &TokenBalance) -> bool {
    if let Some(token_id) = token_balance.token_id {
        list.iter().any(|item| {
            token_balance.address_hash == item.address_hash
                && token_balance.token_contract_address_hash == item.token_contract_address_hash
                && token_id == item.token_id
                && token_balance.block_number == item.block_number
        })
    } else {
        list.iter().any(|item| {
            token_balance.address_hash == item.address_hash
                && token_balance.token_contract_address_hash == item.token_contract_address_hash
                && item.token_id.is_none()
                && token_balance.block_number == item.block_number
        })
    }
}

fn log_fetching_errors(token_balances_params: &[TokenBalance]) {
    let error_messages: Vec<String> = token_balances_params
        .iter()
        .filter(|token_balance| token_balance.error.is_some())
        .map(|token_balance| {
            format!(
                "<address_hash: {}, contract_address_hash: {}, block_number: {}, error: {}>, retried: {} times",
                token_balance.address_hash,
                token_balance.token_contract_address_hash,
                token_balance.block_number,
                token_balance.error.as_ref().unwrap(),
                token_balance.retries_count.unwrap_or(1)
            )
        })
        .collect();

    if !error_messages.is_empty() {
        println!(
            "Errors while fetching TokenBalances through Contract interaction:\n{}",
            error_messages.join("\n")
        );
    }
}

fn unfetched_token_balances(
    token_balances: &[TokenBalance],
    fetched_token_balances: &[TokenBalance],
) -> Vec<TokenBalance> {
    if token_balances.len() == fetched_token_balances.len() {
        Vec::new()
    } else {
        token_balances
            .iter()
            .filter(|token_balance| !present(fetched_token_balances, token_balance))
            .cloned()
            .collect()
    }
}

fn to_address_current_token_balances(address_token_balances: &[TokenBalance]) -> Vec<TokenBalance> {
    let mut token_balances_map: HashMap<String, Vec<TokenBalance>> = HashMap::new();
    for token_balance in address_token_balances {
        let key = format!(
            "{}_{}_{}",
            token_balance.token_contract_address_hash,
            token_balance.token_id.unwrap_or(0),
            token_balance.address_hash
        );
        token_balances_map
            .entry(key)
            .or_insert_with(Vec::new)
            .push(token_balance.clone());
    }

    let mut current_token_balances: Vec<TokenBalance> = Vec::new();
    for (_, grouped_address_token_balances) in token_balances_map {
        let max_block_number = grouped_address_token_balances
            .iter()
            .map(|token_balance| token_balance.block_number)
            .max()
            .unwrap();
        let current_token_balance = grouped_address_token_balances
            .into_iter()
            .find(|token_balance| token_balance.block_number == max_block_number)
            .unwrap();
        current_token_balances.push(current_token_balance);
    }

    current_token_balances.sort_by(|a, b| {
        a.token_contract_address_hash
            .cmp(&b.token_contract_address_hash)
            .then(a.token_id.unwrap_or(0).cmp(&b.token_id.unwrap_or(0)))
            .then(a.address_hash.cmp(&b.address_hash))
    });

    current_token_balances
}
