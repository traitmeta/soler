use std::collections::HashMap;
use std::convert::TryFrom;
use std::str::FromStr;

use ethers::types::{Address, Log as Event, H160, H256, U256};

use crate::common::consts::{self, ERC1155, ERC20, ERC721, WETH};
use crate::pkg::abi;
use entities::{token_transfers::Model as TokenTransfer, tokens::Model as Token};

/// TokenTransfers struct holds the parsed token transfers and tokens.
pub struct TokenTransfers {
    pub tokens: Vec<Token>,
    pub token_transfers: Vec<TokenTransfer>,
}

/// ParseTokenTransfers parses the given logs and returns the token transfers and tokens.
pub fn parse_token_transfers(
    logs: Vec<Event>,
) -> Result<TokenTransfers, Box<dyn std::error::Error>> {
    let mut token_transfers = TokenTransfers {
        tokens: vec![],
        token_transfers: vec![],
    };

    token_transfers = do_parse(logs, token_transfers)?;

    // Filter out supply transfers
    let mut filtered_supply_transfers = HashMap::new();
    for token_transfer in &token_transfers.token_transfers {
        if token_transfer.to_address == consts::ZERO_ADDRESS
            || token_transfer.from_address == consts::ZERO_ADDRESS
        {
            filtered_supply_transfers.insert(token_transfer.token_contract_address.clone(), true);
        }
    }

    let mut unique_tokens = HashMap::new();
    for token in &token_transfers.tokens {
        unique_tokens.insert(token.contract_address.clone(), token.clone());
    }

    // Process total supply for ERC20 tokens
    let mut upsert_tokens = vec![];
    for token in unique_tokens.values() {
        if !filtered_supply_transfers.contains_key(&token.contract_address) {
            continue;
        }

        // if let Ok(total_supply) = abi::get_erc20_total_supply(&token.contract_address) {
        //     let mut token = token.clone();
        //     token.total_supply = total_supply;
        //     upsert_tokens.push(token);
        // }
    }

    token_transfers.tokens = upsert_tokens;

    Ok(token_transfers)
}

fn do_parse(
    logs: Vec<Event>,
    mut acc: TokenTransfers,
) -> Result<TokenTransfers, Box<dyn std::error::Error>> {
    let filtered_logs = filter_logs(logs);

    for (token_type, val) in filtered_logs {
        match token_type.as_str() {
            consts::ERC20 => {
                acc = do_parse_erc20(val, acc)?;
            }
            consts::WETH => {
                acc = do_parse_weth(val, acc)?;
            }
            consts::ERC721 => {
                acc = do_parse_erc721(val, acc)?;
            }
            consts::ERC1155 => {
                acc = do_parse_erc1155(val, acc)?;
            }
            _ => {}
        }
    }

    Ok(acc)
}

fn filter_logs(logs: Vec<Event>) -> HashMap<String, Vec<Event>> {
    let mut filtered_logs = HashMap::new();

    for log in logs {
        if log.first_topic == consts::TOKEN_TRANSFER_SIGNATURE {
            if log.fourth_topic.is_empty() {
                filtered_logs
                    .entry(consts::ERC20.to_string())
                    .or_insert_with(Vec::new)
                    .push(log);
            } else {
                filtered_logs
                    .entry(consts::ERC721.to_string())
                    .or_insert_with(Vec::new)
                    .push(log);
            }
        }

        if log.first_topic == consts::WETH_DEPOSIT_SIGNATURE
            || log.first_topic == consts::WETH_WITHDRAWAL_SIGNATURE
        {
            filtered_logs
                .entry(consts::WETH.to_string())
                .or_insert_with(Vec::new)
                .push(log);
        }

        if log.first_topic == consts::ERC1155_SINGLE_TRANSFER_SIGNATURE
            || log.first_topic == consts::ERC1155_BATCH_TRANSFER_SIGNATURE
        {
            filtered_logs
                .entry(consts::ERC1155.to_string())
                .or_insert_with(Vec::new)
                .push(log);
        }
    }

    filtered_logs
}

fn do_parse_erc721(
    logs: Vec<Event>,
    mut acc: TokenTransfers,
) -> Result<TokenTransfers, Box<dyn std::error::Error>> {
    for log in logs {
        let (token, mut token_transfer) = do_parse_base_token_transfer(log);

        token_transfer.token_id = U256::from_str(&log.fourth_topic)?;
        token_transfer.from_address = Address::from_str(&log.second_topic)?.to_string();
        token_transfer.to_address = Address::from_str(&log.third_topic)?.to_string();

        token.token_type = ERC721.to_string();

        acc.tokens.push(token);
        acc.token_transfers.push(token_transfer);
    }

    Ok(acc)
}

fn do_parse_erc20(
    logs: Vec<Event>,
    mut acc: TokenTransfers,
) -> Result<TokenTransfers, Box<dyn std::error::Error>> {
    for log in logs {
        let (token, mut token_transfer) = do_parse_base_token_transfer(log);

        let amount = abi::parse_erc20_transfer_log(&log.data)?;
        token_transfer.amount = amount;
        token_transfer.from_address = Address::from_str(&log.second_topic)?.to_string();
        token_transfer.to_address = Address::from_str(&log.third_topic)?.to_string();
        token.token_type = ERC20.to_string();

        acc.tokens.push(token);
        acc.token_transfers.push(token_transfer);
    }

    Ok(acc)
}

fn do_parse_weth(
    logs: Vec<Event>,
    mut acc: TokenTransfers,
) -> Result<TokenTransfers, Box<dyn std::error::Error>> {
    for log in logs {
        let (token, mut token_transfer) = do_parse_base_token_transfer(log);

        let amount = abi::parse_erc20_transfer_log(&log.data)?;
        token_transfer.amount = amount;

        if log.first_topic == consts::WETH_DEPOSIT_SIGNATURE {
            token_transfer.from_address = consts::ZERO_ADDRESS.to_string();
            token_transfer.to_address = Address::from_str(&log.second_topic)?.to_string();
        } else {
            token_transfer.from_address = Address::from_str(&log.second_topic)?.to_string();
            token_transfer.to_address = consts::ZERO_ADDRESS.to_string();
        }

        token.token_type = ERC20.to_string();

        acc.tokens.push(token);
        acc.token_transfers.push(token_transfer);
    }

    Ok(acc)
}

fn do_parse_erc1155(
    logs: Vec<Event>,
    mut acc: TokenTransfers,
) -> Result<TokenTransfers, Box<dyn std::error::Error>> {
    for log in logs {
        let (token, mut token_transfer) = do_parse_base_token_transfer(log);

        token_transfer.from_address = Address::from_str(&log.third_topic)?.to_string();
        token_transfer.to_address = Address::from_str(&log.fourth_topic)?.to_string();

        token.token_type = ERC1155.to_string();

        if log.first_topic == consts::ERC1155_SINGLE_TRANSFER_SIGNATURE {
            let (token_id, value) = abi::parse_erc1155_single_transfer_log(&log.data)?;
            token_transfer.token_id = token_id;
            token_transfer.amount = value;
        } else {
            let (token_ids, values) = abi::parse_erc1155_batch_transfer_log(&log.data)?;
            token_transfer.token_ids = token_ids;
            token_transfer.amounts = values;
        }

        acc.tokens.push(token);
        acc.token_transfers.push(token_transfer);
    }

    Ok(acc)
}

fn do_parse_base_token_transfer(log: Event) -> (Token, TokenTransfer) {
    let token_transfer = TokenTransfer {
        block_number: log.block_number,
        block_hash: log.block_hash,
        log_index: log.log_index,
        token_contract_address_hash: log.address.clone(),
        transaction_hash: log.tx_hash,
        ..Default::default()
    };

    let token = Token {
        contract_address_hash: log.address,
        ..Default::default()
    };

    (token, token_transfer)
}
