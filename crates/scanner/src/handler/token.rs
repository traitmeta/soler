use crate::{
    common::{self, consts},
    contracts::{decode, erc20::IERC20Call},
};
use anyhow::{anyhow, Error};
use chrono::Utc;
use entities::token_transfers::Model as TokenTransfersModel;
use entities::tokens::Model as TokenModel;
use ethers::types::{Log, TransactionReceipt, U256};
use repo::dal::token::{Mutation, Query};
use sea_orm::{
    prelude::{BigDecimal, Decimal},
    DbConn,
};
use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, HashSet},
    str::FromStr,
};

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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
struct TokenTransfer {
    amount: String,
    block_number: u64,
    block_hash: String,
    log_index: u64,
    from_address_hash: String,
    to_address_hash: String,
    transaction_hash: String,
    token_contract_address_hash: String,
    token_ids: Option<Vec<String>>,
    token_type: String,
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Serialize, Deserialize)]
struct Token {
    contract_address_hash: String,
    token_type: String,
}
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct Topics {
    first_topic: Option<String>,
    second_topic: Option<String>,
    third_topic: Option<String>,
    fourth_topic: Option<String>,
}

fn get_topics(log: &Log) -> Topics {
    let mut topics = Topics::default();
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
    match topics.second_topic {
        Some(second) => match topics.third_topic {
            Some(third) => match topics.fourth_topic {
                Some(fourth) => TokenKind::Erc721Topic,
                None => TokenKind::ERC20,
            },
            None => match topics.fourth_topic {
                Some(fourth) => TokenKind::None,
                None => TokenKind::Erc20Weth,
            },
        },
        None => match topics.third_topic {
            Some(third) => TokenKind::None,
            None => match topics.fourth_topic {
                Some(fourth) => TokenKind::None,
                None => TokenKind::Erc721Data,
            },
        },
    }
}

fn parse(logs: Vec<Log>) -> (Vec<Token>, Vec<TokenTransfer>) {
    let mut initial_acc = (vec![], vec![]);
    let erc20_and_erc721_token_transfers = logs
        .iter()
        .filter(|log| {
            Some(format!("0x{}", hex::encode(log.topics[0].as_bytes())))
                .is_some_and(|first| first == consts::TOKEN_TRANSFER_SIGNATURE)
        })
        .fold(initial_acc.clone(), |acc, log| {
            do_parse(log, acc.clone(), consts::ERC20)
        });

    let weth_transfers = logs
        .iter()
        .filter(|log| {
            Some(format!("0x{}", hex::encode(log.topics[0].as_bytes()))).is_some_and(|first| {
                first == consts::WETH_DEPOSIT_SIGNATURE
                    || first == consts::WETH_WITHDRAWAL_SIGNATURE
            })
        })
        .fold(initial_acc.clone(), |acc, log| {
            do_parse(log, acc.clone(), consts::ERC20)
        });

    let erc1155_token_transfers = logs
        .iter()
        .filter(|log| {
            Some(format!("0x{}", hex::encode(log.topics[0].as_bytes()))).is_some_and(|first| {
                first == consts::ERC1155_BATCH_TRANSFER_SIGNATURE
                    || first == consts::ERC1155_SINGLE_TRANSFER_SIGNATURE
            })
        })
        .fold(initial_acc.clone(), |acc, log| {
            do_parse(log, acc.clone(), consts::ERC1155)
        });

    let rough_tokens = [
        erc1155_token_transfers.0,
        erc20_and_erc721_token_transfers.0,
        weth_transfers.0,
    ]
    .concat();

    let rough_token_transfers = [
        erc1155_token_transfers.1,
        erc20_and_erc721_token_transfers.1,
        weth_transfers.1,
    ]
    .concat();

    let (tokens, token_transfers) = sanitize_token_types(rough_tokens, rough_token_transfers);

    let token_transfers_filtered = token_transfers
        .into_iter()
        .filter(|token_transfer| {
            token_transfer.to_address_hash == consts::BURN_ADDRESS
                || token_transfer.from_address_hash == consts::BURN_ADDRESS
        })
        .collect::<Vec<TokenTransfer>>();

    let token_contract_addresses = token_transfers_filtered
        .iter()
        .map(|token_transfer| token_transfer.token_contract_address_hash.clone())
        .collect::<Vec<String>>();

    let unique_token_contract_addresses = token_contract_addresses
        .iter()
        .cloned()
        .collect::<HashSet<String>>();

    // TokenTotalSupplyUpdater::add_tokens(unique_token_contract_addresses);
    let tokens_uniq = tokens.iter().cloned().collect::<HashSet<Token>>();

    let token_transfers = token_transfers_filtered.into_iter().clone().collect();

    (tokens_uniq.into_iter().collect(), token_transfers)
}

fn sanitize_token_types(
    tokens: Vec<Token>,
    token_transfers: Vec<TokenTransfer>,
) -> (Vec<Token>, Vec<TokenTransfer>) {
    let existing_token_types_map = HashMap::new();

    // TODO query from db
    // for token in tokens {
    //     match Repo.get_by(Token, contract_address_hash: token.contract_address_hash) {
    //         Some(existing_token) => {
    //             existing_token_types_map.insert(token.contract_address_hash, existing_token.type);
    //         }
    //         None => {}
    //     }
    // }

    let existing_tokens = existing_token_types_map
        .keys()
        .cloned()
        .collect::<Vec<String>>();

    let new_tokens_token_transfers = token_transfers
        .iter()
        .filter(|token_transfer| {
            !existing_tokens.contains(&token_transfer.token_contract_address_hash)
        })
        .cloned()
        .collect::<Vec<TokenTransfer>>();

    let mut new_token_map = HashMap::new();
    new_tokens_token_transfers.iter().for_each(|t| {
        new_token_map
            .entry(t.token_contract_address_hash.clone())
            .or_insert_with(Vec::new)
            .push(t.clone())
    });
    let new_token_types_map = new_token_map
        .iter()
        .map(|(contract_address_hash, transfers)| {
            (
                contract_address_hash.clone(),
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
            token_transfer.token_type =
                actual_token_types_map[&token_transfer.token_contract_address_hash].clone();
            token_transfer
        })
        .collect::<Vec<TokenTransfer>>();

    (actual_tokens, actual_token_transfers)
}

fn define_token_type(token_transfers: Vec<TokenTransfer>) -> String {
    token_transfers
        .iter()
        .fold(None, |acc: Option<String>, token_transfer| match acc {
            Some(token_type) => {
                if token_type_priority(token_transfer.token_type.clone())
                    > token_type_priority(token_type.clone())
                {
                    Some(token_transfer.token_type.clone())
                } else {
                    Some(token_type)
                }
            }
            None => Some(token_transfer.token_type.clone()),
        })
        .unwrap_or_else(|| "-1".to_string())
}

fn token_type_priority(token_type: String) -> i32 {
    let token_types_priority_order = [consts::ERC20, consts::ERC721, consts::ERC1155];
    token_types_priority_order
        .iter()
        .position(|&t| t == token_type)
        .unwrap_or(usize::MAX) as i32
}

fn do_parse(
    log: &Log,
    mut acc: (Vec<Token>, Vec<TokenTransfer>),
    token_type: &str,
) -> (Vec<Token>, Vec<TokenTransfer>) {
    let (token, token_transfer) = match token_type {
        // consts::ERC1155 => parse_erc1155_params(log),
        consts::ERC20 => parse_erc20_params(log),
        consts::ERC721 => parse_erc721_params(log),
        consts::WETH => parse_weth_params(log),
        _ => unreachable!(),
    };

    acc.0.push(token);
    acc.1.push(token_transfer);

    acc
}

fn parse_erc20_params(log: &Log) -> (Token, TokenTransfer) {
    let topics = get_topics(log);
    let amount = match decode::decode_erc20_event_data(log.data.to_vec().as_slice()) {
        Ok(value) => value,
        Err(err) => {
            tracing::info!(message = "parse_erc20_params", err = ?err);
            U256::from(0)
        }
    };

    let token_transfer = TokenTransfer {
        amount: amount.to_string(),
        block_number: log.block_number.map_or(0, |number| number.as_u64()),
        block_hash: log.block_hash.map_or("".to_string(), |hash| {
            String::from_utf8(hash.as_bytes().to_vec()).unwrap()
        }),
        log_index: log.log_index.map_or(0, |idx| idx.as_u64()),
        from_address_hash: truncate_address_hash(topics.second_topic),
        to_address_hash: truncate_address_hash(topics.third_topic),
        token_contract_address_hash: String::from_utf8(log.address.as_bytes().to_vec()).unwrap(),
        token_ids: None,
        token_type: consts::ERC20.to_string(),
        transaction_hash: log.transaction_hash.map_or("".to_string(), |hash| {
            String::from_utf8(hash.as_bytes().to_vec()).unwrap()
        }),
    };

    let token = Token {
        contract_address_hash: String::from_utf8(log.address.as_bytes().to_vec()).unwrap(),
        token_type: consts::ERC20.to_string(),
    };

    (token, token_transfer)
}

fn parse_weth_params(log: &Log) -> (Token, TokenTransfer) {
    let topics = get_topics(log);
    let amount = match decode::decode_erc20_event_data(log.data.to_vec().as_slice()) {
        Ok(value) => value,
        Err(err) => {
            tracing::info!(message = "parse_erc20_params", err = ?err);
            U256::from(0)
        }
    };

    let mut token_transfer = TokenTransfer {
        amount: amount.to_string(),
        block_number: log.block_number.map_or(0, |number| number.as_u64()),
        block_hash: log.block_hash.map_or("".to_string(), |hash| {
            String::from_utf8(hash.as_bytes().to_vec()).unwrap()
        }),
        log_index: log.log_index.map_or(0, |idx| idx.as_u64()),
        from_address_hash: consts::ZERO_ADDRESS.to_string(),
        to_address_hash: consts::ZERO_ADDRESS.to_string(),
        token_contract_address_hash: String::from_utf8(log.address.as_bytes().to_vec()).unwrap(),
        token_ids: None,
        token_type: consts::ERC20.to_string(),
        transaction_hash: log.transaction_hash.map_or("".to_string(), |hash| {
            String::from_utf8(hash.as_bytes().to_vec()).unwrap()
        }),
    };

    if let Some(first_topic) = topics.first_topic {
        if first_topic == consts::WETH_DEPOSIT_SIGNATURE {
            token_transfer.from_address_hash = consts::ZERO_ADDRESS.to_string();
            token_transfer.to_address_hash = truncate_address_hash(topics.second_topic);
        } else {
            token_transfer.from_address_hash = truncate_address_hash(topics.second_topic);
            token_transfer.to_address_hash = consts::ZERO_ADDRESS.to_string();
        }
    }

    let token = Token {
        contract_address_hash: String::from_utf8(log.address.as_bytes().to_vec()).unwrap(),
        token_type: consts::ERC20.to_string(),
    };

    (token, token_transfer)
}

fn parse_erc721_params(log: &Log) -> (Token, TokenTransfer) {
    let topics = get_topics(log);

    let mut token_transfer = TokenTransfer {
        amount: "".to_string(),
        block_number: log.block_number.map_or(0, |number| number.as_u64()),
        block_hash: log.block_hash.map_or("".to_string(), |hash| {
            String::from_utf8(hash.as_bytes().to_vec()).unwrap()
        }),
        log_index: log.log_index.map_or(0, |idx| idx.as_u64()),
        from_address_hash: truncate_address_hash(topics.second_topic.clone()),
        to_address_hash: truncate_address_hash(topics.third_topic),
        token_contract_address_hash: String::from_utf8(log.address.as_bytes().to_vec()).unwrap(),
        token_ids: None,
        token_type: consts::ERC721.to_string(),
        transaction_hash: log.transaction_hash.map_or("".to_string(), |hash| {
            String::from_utf8(hash.as_bytes().to_vec()).unwrap()
        }),
    };

    if topics.second_topic.is_some() {
        match decode::decode_erc20_event_data(log.data.to_vec().as_slice()) {
            Ok(value) => {
                token_transfer.token_ids = Some(vec![value.to_string()]);
            }
            Err(err) => {
                tracing::info!(message = "parse_erc20_params", err = ?err);
            }
        };
    } else {
        match decode::decode_erc721_event_data(log.data.to_vec().as_slice()) {
            Ok((from, to, token_id)) => {
                token_transfer.from_address_hash = hex::encode(from.as_bytes());
                token_transfer.to_address_hash = hex::encode(to.as_bytes());
                token_transfer.token_ids = Some(vec![token_id.to_string()]);
            }
            Err(err) => {
                tracing::info!(message = "parse_erc20_params", err = ?err);
            }
        };
    }

    let token = Token {
        contract_address_hash: String::from_utf8(log.address.as_bytes().to_vec()).unwrap(),
        token_type: consts::ERC721.to_string(),
    };

    (token, token_transfer)
}

fn parse_erc1155_params(log: &Log) -> (TokenModel, TokenTransfersModel) {
    let topics = get_topics(log);

    let mut transfer_model = TokenTransfersModel {
        transaction_hash: log
            .transaction_hash
            .map_or(vec![], |hash| hash.as_bytes().to_vec()),
        log_index: log.log_index.map_or(0, |idx| idx.as_u32() as i32),
        from_address_hash: topics
            .third_topic
            .map_or(consts::ZERO_ADDRESS.as_bytes().to_vec(), |topic| {
                topic.into_bytes()
            }),
        to_address_hash: topics
            .fourth_topic
            .map_or(consts::ZERO_ADDRESS.as_bytes().to_vec(), |topic| {
                topic.into_bytes()
            }),
        amount: None,
        token_id: None,
        token_contract_address_hash: log.address.as_bytes().to_vec(),
        inserted_at: Utc::now().naive_utc(),
        updated_at: Utc::now().naive_utc(),
        block_number: log.block_number.map(|number| number.as_u32() as i32),
        block_hash: log
            .block_hash
            .map_or(vec![], |hash| hash.as_bytes().to_vec()),
        amounts: None,
        token_ids: None,
    };

    let token = TokenModel {
        contract_address_hash: log.address.as_bytes().to_vec(),
        name: None,
        symbol: None,
        total_supply: None,
        decimals: None,
        r#type: consts::ERC1155.to_string(),
        cataloged: None,
        inserted_at: Utc::now().naive_utc(),
        updated_at: Utc::now().naive_utc(),
        holder_count: None,
        skip_metadata: None,
        fiat_value: None,
        circulating_market_cap: None,
        total_supply_updated_at_block: None,
        icon_url: None,
        is_verified_via_admin_panel: None,
    };

    if let Some(first_topic) = topics.first_topic {
        if first_topic == consts::ERC1155_BATCH_TRANSFER_SIGNATURE {
            match decode::decode_erc1155_batch_event_data(log.data.to_vec().as_slice()) {
                Ok((ids, values)) => {
                    transfer_model.amounts = Some(
                        values
                            .iter()
                            .map(|val| BigDecimal::from_str(val.to_string().as_str()).unwrap())
                            .collect(),
                    );
                    transfer_model.token_ids = Some(
                        ids.iter()
                            .map(|id| BigDecimal::from_str(id.to_string().as_str()).unwrap())
                            .collect(),
                    );
                }
                Err(err) => {
                    tracing::info!(message = "parse_erc1155_params", err = ?err);
                }
            };
        } else {
            match decode::decode_erc1155_single_event_data(log.data.to_vec().as_slice()) {
                Ok((id, value)) => {
                    transfer_model.amount =
                        Some(BigDecimal::from_str(value.to_string().as_str()).unwrap());
                    transfer_model.token_id =
                        Some(BigDecimal::from_str(id.to_string().as_str()).unwrap());
                }
                Err(err) => {
                    tracing::info!(message = "parse_erc1155_params", err = ?err);
                }
            };
        }
    }

    (token, transfer_model)
}

fn truncate_address_hash(address_hash: Option<String>) -> String {
    match address_hash {
        Some(hash) => {
            if let Some(stripped) = hash.strip_prefix("0x000000000000000000000000") {
                format!("0x{}", stripped)
            } else {
                hash
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
        offset += (size / 8) as usize;
    }

    decoded_values
}
