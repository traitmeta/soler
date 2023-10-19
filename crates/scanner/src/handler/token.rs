use crate::{common::{consts, self}, contracts::erc20::IERC20Call};
use anyhow::{anyhow, Error};
use ethers::types::{Log, TransactionReceipt, U256};
use repo::dal::token::{Mutation, Query};
use sea_orm::{prelude::Decimal, DbConn};
use std::collections::{HashMap, HashSet};
use serde::{Deserialize, Serialize};

pub struct TokenHandler {
    rpc_url: String,
}

impl TokenHandler {
    pub fn new(rpc_url: &str) -> TokenHandler {
        Self {
            rpc_url: rpc_url.to_string(),
        }
    }

    pub async fn handle_erc20_metadata(&self, conn: &DbConn) -> Result<(), Error> {
        let erc20_call = IERC20Call::new(self.rpc_url.as_str());
        match Query::filter_not_skip_metadata(conn, consts::ERC20).await {
            Ok(models) => {
                for mut model in models.into_iter() {
                    let contract_addr = std::str::from_utf8(&model.contract_address_hash).unwrap();
                    if let Ok((name, symbol, decimals)) = erc20_call.metadata(contract_addr).await {
                        model.name = Some(name);
                        model.symbol = Some(symbol);
                        model.decimals = Some(Decimal::new(decimals as i64, 0));
                    }

                    if let Err(e) = Mutation::update_metadata(conn, &model).await {
                        return Err(anyhow!("Handler Erc20 metadata: {:?}", e.to_string()));
                    }
                }
                Ok(())
            }
            Err(e) => Err(anyhow!("Handler Erc20 metadata: {:?}", e.to_string())),
        }
    }
}

struct TokenTransfer {
    amount: f64,
    block_number: u64,
    block_hash: String,
    log_index: u64,
    from_address_hash: String,
    to_address_hash: String,
    transaction_hash: String,
    token_contract_address_hash: String,
    token_ids: Option<Vec<u64>>,
    token_type: String,
}

struct Token {
    contract_address_hash: String,
    token_type: String,
}

fn process_token_transfers(receipts: &[TransactionReceipt]) ->HashMap<String, Vec<TokenTransfer>> {
    let mut initial_acc = HashMap::new(); 
    
    initial_acc 
}

fn parse(logs: Vec<Log>) -> HashMap<String, Vec<TokenTransfer>> {
    let mut initial_acc = HashMap::new();
    initial_acc.insert("tokens".to_string(), Vec::new());
    initial_acc.insert("token_transfers".to_string(), Vec::new());

    let erc20_and_erc721_token_transfers = logs
        .iter()
        .filter(|log| {
            log.first_topic == consts::TOKEN_TRANSFER_SIGNATURE()
        })
        .fold(initial_acc.clone(), |acc, log| do_parse(log, acc.clone(),consts::ERC20));

    let weth_transfers = logs
        .iter()
        .filter(|log| {
            log.first_topic == consts::WETH_DEPOSIT_SIGNATURE()
                || log.first_topic == consts::WETH_WITHDRAWAL_SIGNATURE
        })
        .fold(initial_acc.clone(), |acc, log| do_parse(log, acc.clone(),consts::ERC20));

    let erc1155_token_transfers = logs
        .iter()
        .filter(|log| {
            log.first_topic == consts::ERC1155_BATCH_TRANSFER_SIGNATURE
                || log.first_topic == consts::ERC1155_SINGLE_TRANSFER_SIGNATURE
        })
        .fold(initial_acc.clone(), |acc, log| do_parse(log, acc.clone(), consts::ERC1155));

    let rough_tokens = [
        erc1155_token_transfers.tokens,
        erc20_and_erc721_token_transfers.tokens,
        weth_transfers.tokens,
    ]
    .concat();

    let rough_token_transfers = [
        erc1155_token_transfers.token_transfers,
        erc20_and_erc721_token_transfers.token_transfers,
        weth_transfers.token_transfers,
    ]
    .concat();

    let (tokens, token_transfers) = sanitize_token_types(rough_tokens, rough_token_transfers);

    let token_transfers_filtered = token_transfers
        .iter()
        .filter(|token_transfer| {
            token_transfer.to_address_hash == consts::BURN_ADDRESS
                || token_transfer.from_address_hash == consts::BURN_ADDRESS
        })
        .collect::<Vec<&TokenTransfer>>();

    let token_contract_addresses = token_transfers_filtered
        .iter()
        .map(|token_transfer| token_transfer.token_contract_address_hash.clone())
        .collect::<Vec<String>>();

    let unique_token_contract_addresses = token_contract_addresses.iter().cloned().collect::<HashSet<String>>();

    TokenTotalSupplyUpdater::add_tokens(unique_token_contract_addresses);

    let tokens_uniq = tokens.iter().cloned().collect::<HashSet<Token>>();

    let token_transfers_from_logs_uniq = TokenTransfersFromLogs {
        tokens: tokens_uniq.into_iter().collect(),
        token_transfers: token_transfers_filtered.into_iter().collect(),
    };

    token_transfers_from_logs_uniq
}

fn sanitize_token_types(
    tokens: Vec<Token>,
    token_transfers: Vec<TokenTransfer>,
) -> (Vec<Token>, Vec<TokenTransfer>) {
    let mut existing_token_types_map = HashMap::new();

    for token in tokens {
        match Repo.get_by(Token, contract_address_hash: token.contract_address_hash) {
            Some(existing_token) => {
                existing_token_types_map.insert(token.contract_address_hash, existing_token.type);
            }
            None => {}
        }
    }

    let existing_tokens = existing_token_types_map.keys().cloned().collect::<Vec<String>>();

    let new_tokens_token_transfers = token_transfers
        .iter()
        .filter(|token_transfer| !existing_tokens.contains(&token_transfer.token_contract_address_hash))
        .cloned()
        .collect::<Vec<TokenTransfer>>();

    let new_token_types_map = new_tokens_token_transfers
        .iter()
        .group_by(|token_transfer| token_transfer.token_contract_address_hash.clone())
        .map(|(contract_address_hash, transfers)| {
            (
                contract_address_hash,
                define_token_type(transfers.clone()),
            )
        })
        .collect::<HashMap<String, String>>();

    let actual_token_types_map = new_token_types_map
        .into_iter()
        .chain(existing_token_types_map)
        .collect::<HashMap<String, String>>();

    let actual_tokens = tokens
        .iter()
        .map(|token| {
            let mut token = token.clone();
            token.token_type = actual_token_types_map[&token.contract_address_hash].clone();
            token
        })
        .collect::<Vec<Token>>();

    let actual_token_transfers = token_transfers
        .iter()
        .map(|token_transfer| {
            let mut token_transfer = token_transfer.clone();
            token_transfer.token_type = actual_token_types_map[&token_transfer.token_contract_address_hash].clone();
            token_transfer
        })
        .collect::<Vec<TokenTransfer>>();

    (actual_tokens, actual_token_transfers)
}

fn define_token_type(token_transfers: Vec<TokenTransfer>) -> String {
    token_transfers
        .iter()
        .fold(None, |acc, token_transfer| {
            match acc {
                Some(token_type) => {
                    if token_type_priority(&token_transfer.token_type) > token_type_priority(&token_type) {
                        Some(token_transfer.token_type.clone())
                    } else {
                        Some(token_type)
                    }
                }
                None => Some(token_transfer.token_type.clone()),
            }
        })
        .unwrap_or_else(|| "-1".to_string())
}

fn token_type_priority(token_type: &str) -> i32 {
    let token_types_priority_order = vec!["ERC-20", "ERC-721", "ERC-1155"];
    token_types_priority_order
        .iter()
        .position(|&t| t == token_type)
        .unwrap_or(-1) as i32
}

fn do_parse(log: &Log, acc: HashMap<String, Vec<TokenTransfer>>, token_type: &str) -> HashMap<String, Vec<TokenTransfer>> {
    let (token, token_transfer) = if token_type != consts::ERC1155 {
        parse_params(log)
    } else {
        parse_erc1155_params(log)
    };

    let mut tokens = acc.get("tokens").unwrap().clone();
    tokens.push(token);

    let mut token_transfers = acc.get("token_transfers").unwrap().clone();
    token_transfers.push(token_transfer);

    let mut new_acc = HashMap::new();
    new_acc.insert("tokens".to_string(), tokens);
    new_acc.insert("token_transfers".to_string(), token_transfers);

    new_acc
}

struct Topics {
    first_topic: Option<String>,
    second_topic: Option<String>,
    third_topic: Option<String>,
    fourth_topic: Option<String>,
}

fn get_topics(log: &Log) -> Topics {
    let mut topics: Topics;
    for (i, topic) in log.topics.iter().enumerate() {
        let tp = Some(format!("0x{}", hex::encode(topic.as_bytes())));
        match i {
            0 => topics.first_topic = tp,
            1 => topics.second_topic = tp,
            2 => topics.third_topic = tp,
            3 => topics.fourth_topic = tp,
            _ => (),
        }
    }
    
    topics
}


#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub enum TokenKind {
    #[serde(rename = "ERC-20")]
    ERC20,
    #[serde(rename = "ERC-20")]
    Erc20Weth,
    #[serde(rename = "ERC-721")]
    Erc721Topic,
    #[serde(rename = "ERC-721")]
    Erc721Data,
    #[serde(rename = "ERC-1155")]
    ERC1155,
    None,
}

/*
    -------------------------------------------------------------------------------------
    |   second topic   |   third topic   |   fourth topic   |     data    |   result    |
    -------------------------------------------------------------------------------------
    |       Some       |       Some      |       None       |  Some/None  |   ERC-20    |
    -------------------------------------------------------------------------------------
    |       Some       |       None      |       None       |  Some/None  | ERC-20/WETH |
    -------------------------------------------------------------------------------------
    |       Some       |       Some      |       Some       |  Some/None  |   ERC-721   |
    ------------------------------------------------------------------------------------- 
    |       None       |       None      |       None       |     Some    |   ERC-721   |
    ------------------------------------------------------------------------------------- 
*/

fn match_token_type(log: &Log) -> TokenKind {
    let topics = get_topics(log);
    match topics.second_topic{
        Some(second) => match topics.third_topic{
            Some(third) => match topics.fourth_topic{
                Some(fourth) =>  TokenKind::Erc721Topic,
                None => TokenKind::ERC20,
            },
            None => match topics.fourth_topic{
                Some(fourth) => TokenKind::None,
                None => TokenKind::Erc20Weth,
            }
        },
        None => match topics.third_topic{
            Some(third) => TokenKind::None,
            None => match topics.fourth_topic{
                Some(fourth) => TokenKind::None,
                None => TokenKind::Erc721Data,
            }
        },
    }
}

fn parse_params(log: &Log) -> (Option<Token>, Option<TokenTransfer>) {
    if let Some(second_topic) = log.second_topic {
        if let Some(third_topic) = log.third_topic {
            if log.fourth_topic.is_none() {
                let amount = decode_data(log.data, vec![("uint", 256)])[0];

                let token_transfer = TokenTransfer {
                    amount: amount as f64,
                    block_number: log.block_number,
                    block_hash: log.block_hash.clone(),
                    log_index: log.index,
                    from_address_hash: truncate_address_hash(log.second_topic.clone()),
                    to_address_hash: truncate_address_hash(log.third_topic.clone()),
                    token_contract_address_hash: log.address_hash.clone(),
                    token_ids: None,
                    token_type: consts::ERC20.to_string(),
                    transaction_hash: log.transaction_hash.clone(),
                };

                let token = Token {
                    contract_address_hash: log.address_hash.clone(),
                    token_type: consts::ERC20.to_string(),
                };

                return (Some(token), Some(token_transfer));
            }
        }
    }

    (None, None)
}


fn parse_erc721_params_with_topic(log: &Log) -> (Token, TokenTransfer) {
    let amount = decode_data(log.fourth_topic, vec![("uint", 256)])[0];
    let token_transfer = TokenTransfer {
        amount: amount as f64,
        block_number: log.block_number,
        block_hash: log.block_hash.clone(),
        log_index: log.index,
        from_address_hash: truncate_address_hash(log.second_topic.clone()),
        to_address_hash: truncate_address_hash(log.third_topic.clone()),
        token_contract_address_hash: log.address_hash.clone(),
        token_ids: None,
        token_type: consts::ERC721.to_string(),
        transaction_hash: log.transaction_hash.clone(),
    };

    let token = Token {
        contract_address_hash: log.address_hash.clone(),
        token_type: consts::ERC721.to_string(),
    };

    (token, token_transfer)
}

fn parse_erc721_params_with_data(log: &Log) -> (Token, TokenTransfer) {
    let (from_address_hash, to_address_hash, token_id) = decode_data(log.data, vec!["address", "address", ("uint", 256)])[0];
    let token_transfer = TokenTransfer {
        amount: amount as f64,
        block_number: log.block_number,
        block_hash: log.block_hash.clone(),
        log_index: log.index,
        from_address_hash: truncate_address_hash(from_address_hash.clone()),
        to_address_hash: truncate_address_hash(to_address_hash.clone()),
        token_contract_address_hash: log.address_hash.clone(),
        token_ids: Some(vec![token_id]),
        token_type: consts::ERC721.to_string(),
        transaction_hash: log.transaction_hash.clone(),
    };

    let token = Token {
        contract_address_hash: log.address_hash.clone(),
        token_type: consts::ERC721.to_string(),
    };

    (token, token_transfer)
}



fn parse_erc1155_params(log: &Log) -> (Option<Token>, Option<TokenTransfer>) {
    if log.first_topic == consts::ERC1155_BATCH_TRANSFER_SIGNATURE{
        if let Some(third_topic) = log.third_topic {
            if let Some(fourth_topic) = log.fourth_topic {
                let (token_ids, values) = decode_data(log.data, vec![("array", ("uint", 256)), ("array", ("uint", 256))]);

                let token_transfer = TokenTransfer {
                    block_number: log.block_number,
                    block_hash: log.block_hash.clone(),
                    log_index: log.index,
                    from_address_hash: truncate_address_hash(third_topic.clone()),
                    to_address_hash: truncate_address_hash(fourth_topic.clone()),
                    token_contract_address_hash: log.address_hash.clone(),
                    transaction_hash: log.transaction_hash.clone(),
                    token_type: consts::ERC1155.to_string(),
                    token_ids: Some(token_ids),
                    amount: values,
                };

                let token = Token {
                    contract_address_hash: log.address_hash.clone(),
                    token_type: consts::ERC1155.to_string(),
                };

                return (Some(token), Some(token_transfer));
            }
        }
    }

    (None, None)
}

fn truncate_address_hash(address_hash: Option<String>) -> String {
    match address_hash {
        Some(hash) => {
            if hash.starts_with("0x000000000000000000000000") {
                return format!("0x{}", &hash[26..]);
            }
        }
        None => "0x0000000000000000000000000000000000000000".to_string(),
    }
}

fn encode_address_hash(binary: Vec<u8>) -> String {
    format!("0x{}", hex::encode(binary))
}

fn decode_data(encoded_data: &str, types: Vec<(&str, u32)>) -> Vec<u64> {
    if encoded_data == "0x" {
        return vec![0; types.len()];
    }

    let decoded_data = hex::decode(&encoded_data[2..]).unwrap();

    let mut offset = 0;
    let mut decoded_values = Vec::new();

    for (data_type, size) in types {
        let value = match data_type {
            "uint" => {
                let mut bytes = [0; 32];
                bytes.copy_from_slice(&decoded_data[offset..offset + 32]);
                U256::from_little_endian(&bytes).as_u64()
            }
            // Other data types...
            _ => 0,
        };

        decoded_values.push(value);
        offset += size / 8;
    }

    decoded_values
}


