use crate::{common::consts, contracts::decode};
use chrono::Utc;
use entities::token_transfers::Model as TokenTransferModel;
use entities::tokens::Model as TokenModel;
use ethers::types::{Log, TransactionReceipt, H160, H256};
use sea_orm::prelude::BigDecimal;
use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, HashSet},
    str::FromStr,
};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct DecodedTopics {
    first_topic: Option<H256>,
    second_topic: Option<H256>,
    third_topic: Option<H256>,
    fourth_topic: Option<H256>,
}

fn decode_topics(log: &Log) -> DecodedTopics {
    let mut topics = DecodedTopics::default();
    for (i, topic) in log.topics.iter().enumerate() {
        match i {
            0 => topics.first_topic = Some(*topic),
            1 => topics.second_topic = Some(*topic),
            2 => topics.third_topic = Some(*topic),
            3 => topics.fourth_topic = Some(*topic),
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

pub fn handle_token_from_receipts(
    receipts: &[TransactionReceipt],
) -> (Vec<TokenModel>, Vec<TokenTransferModel>) {
    let mut acc: (Vec<TokenModel>, Vec<TokenTransferModel>) = (vec![], vec![]);
    for receipt in receipts.iter() {
        let (mut tokens, mut token_transfers) = token_process(&receipt.logs);
        acc.0.append(&mut tokens);
        acc.1.append(&mut token_transfers);
    }

    let mut tokens_map: HashMap<Vec<u8>, TokenModel> = HashMap::new();
    for token in acc.0 {
        tokens_map
            .entry(token.contract_address_hash.clone())
            .and_modify(|t| t.r#type = confirm_token_type(token.r#type.clone(), t.r#type.clone()))
            .or_insert(token.clone());
    }
    let tokens_uniq = tokens_map.values().cloned().collect::<Vec<TokenModel>>();

    (tokens_uniq, acc.1)
}

pub fn token_process(logs: &[Log]) -> (Vec<TokenModel>, Vec<TokenTransferModel>) {
    let mut acc = (vec![], vec![]);
    for log in logs {
        acc = do_parse(log, acc);
    }

    let mut tokens_map: HashMap<Vec<u8>, TokenModel> = HashMap::new();
    for token in acc.0 {
        tokens_map
            .entry(token.contract_address_hash.clone())
            .and_modify(|t| t.r#type = confirm_token_type(token.r#type.clone(), t.r#type.clone()))
            .or_insert(token.clone());
    }
    let tokens_uniq = tokens_map.values().cloned().collect::<Vec<TokenModel>>();

    (tokens_uniq, acc.1)
}

#[allow(dead_code)]
fn call_update_total_supply(transfers: &[TokenTransferModel]) {
    let burn_transfers = transfers
        .iter()
        .filter(|transfer| {
            transfer.to_address_hash == consts::BURN_ADDRESS.as_bytes().to_vec()
                || transfer.from_address_hash == consts::BURN_ADDRESS.as_bytes().to_vec()
        })
        .cloned()
        .collect::<Vec<TokenTransferModel>>();

    let burn_contract_addresses = burn_transfers
        .iter()
        .map(|transfer| transfer.token_contract_address_hash.clone())
        .collect::<Vec<Vec<u8>>>();

    let _unique_burn_contract_addresses = burn_contract_addresses
        .iter()
        .cloned()
        .collect::<HashSet<Vec<u8>>>();

    // TokenTotalSupplyUpdater::add_tokens(unique_token_contract_addresses);
}

fn confirm_token_type(new_type: String, old_type: String) -> String {
    if token_type_priority(new_type.as_str()) > token_type_priority(old_type.as_str()) {
        return new_type;
    }
    old_type
}

fn token_type_priority(token_type: &str) -> i32 {
    let token_types_priority_order = [consts::ERC20, consts::ERC721, consts::ERC1155];
    token_types_priority_order
        .iter()
        .position(|&t| t == token_type)
        .unwrap_or(usize::MAX) as i32
}

fn do_parse(
    log: &Log,
    mut acc: (Vec<TokenModel>, Vec<TokenTransferModel>),
) -> (Vec<TokenModel>, Vec<TokenTransferModel>) {
    let decoded_topics = decode_topics(log);
    match decoded_topics.first_topic {
        Some(first) => {
            let first_topic = format!("0x{}", hex::encode(first.as_bytes()));
            let first_str = first_topic.as_str();
            let mut kind = TokenKind::None;
            if first_str == consts::TOKEN_TRANSFER_SIGNATURE
                || first_str == consts::WETH_DEPOSIT_SIGNATURE
                || first_str == consts::WETH_WITHDRAWAL_SIGNATURE
            {
                kind = decode_token_type(decoded_topics);
            } else if first_str == consts::ERC1155_BATCH_TRANSFER_SIGNATURE
                || first_str == consts::ERC1155_SINGLE_TRANSFER_SIGNATURE
            {
                kind = TokenKind::ERC1155;
            }
            let (token, token_transfer) = match kind {
                TokenKind::ERC1155 => parse_erc1155_params(log),
                TokenKind::ERC20 => parse_erc20_params(log),
                TokenKind::Erc721Data | TokenKind::Erc721Topic => parse_erc721_params(log),
                TokenKind::Erc20Weth => parse_weth_params(log),
                _ => return acc,
            };

            acc.0.push(token);
            acc.1.push(token_transfer);

            acc
        }
        None => acc,
    }
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
fn decode_token_type(topics: DecodedTopics) -> TokenKind {
    match topics.second_topic {
        Some(_second) => match topics.third_topic {
            Some(_third) => match topics.fourth_topic {
                Some(_fourth) => TokenKind::Erc721Topic,
                None => TokenKind::ERC20,
            },
            None => match topics.fourth_topic {
                Some(_fourth) => TokenKind::None,
                None => TokenKind::Erc20Weth,
            },
        },
        None => match topics.third_topic {
            Some(_third) => TokenKind::None,
            None => match topics.fourth_topic {
                Some(_fourth) => TokenKind::None,
                None => TokenKind::Erc721Data,
            },
        },
    }
}

fn parse_erc20_params(log: &Log) -> (TokenModel, TokenTransferModel) {
    let topics = decode_topics(log);

    let (mut token, mut transfer_model) = defualt_model(log);
    transfer_model.from_address_hash = topics
        .second_topic
        .map_or(consts::ZERO_ADDRESS.as_bytes().to_vec(), |topic| {
            H160::from(topic).as_bytes().to_vec()
        });
    transfer_model.to_address_hash = topics
        .third_topic
        .map_or(consts::ZERO_ADDRESS.as_bytes().to_vec(), |topic| {
            H160::from(topic).as_bytes().to_vec()
        });
    token.r#type = consts::ERC20.to_string();

    match decode::decode_erc20_event_data(log.data.to_vec().as_slice()) {
        Ok(value) => {
            transfer_model.amount = Some(BigDecimal::from_str(value.to_string().as_str()).unwrap())
        }
        Err(err) => {
            tracing::info!(message = "parse_erc20_params", err = ?err);
        }
    };

    (token, transfer_model)
}

fn parse_weth_params(log: &Log) -> (TokenModel, TokenTransferModel) {
    let topics = decode_topics(log);
    let (mut token, mut transfer_model) = defualt_model(log);
    token.r#type = consts::ERC20.to_string();

    match decode::decode_erc20_event_data(log.data.to_vec().as_slice()) {
        Ok(value) => {
            transfer_model.amount = Some(BigDecimal::from_str(value.to_string().as_str()).unwrap());
        }
        Err(err) => {
            tracing::info!(message = "parse_weth_params", err = ?err);
        }
    };

    if let Some(first_topic) = topics.first_topic {
        let first_topic = format!("0x{}", hex::encode(first_topic.as_bytes()));
        let first_str = first_topic.as_str();
        if first_str == consts::WETH_DEPOSIT_SIGNATURE {
            // TODO this will be mint for api
            if let Some(second_topic) = topics.second_topic {
                transfer_model.to_address_hash = H160::from(second_topic).as_bytes().to_vec();
            }
        } else if let Some(second_topic) = topics.second_topic {
            transfer_model.from_address_hash = H160::from(second_topic).as_bytes().to_vec();
        }
    }

    (token, transfer_model)
}

fn parse_erc721_params(log: &Log) -> (TokenModel, TokenTransferModel) {
    let topics = decode_topics(log);
    let (mut token, mut transfer_model) = defualt_model(log);
    transfer_model.from_address_hash = topics
        .second_topic
        .map_or(consts::ZERO_ADDRESS.as_bytes().to_vec(), |topic| {
            H160::from(topic).as_bytes().to_vec()
        });
    transfer_model.to_address_hash = topics
        .third_topic
        .map_or(consts::ZERO_ADDRESS.as_bytes().to_vec(), |topic| {
            H160::from(topic).as_bytes().to_vec()
        });
    token.r#type = consts::ERC721.to_string();

    match topics.fourth_topic {
        Some(fourth_topic) => match decode::decode_erc20_event_data(fourth_topic.as_bytes()) {
            Ok(value) => {
                transfer_model.token_id =
                    Some(BigDecimal::from_str(value.to_string().as_str()).unwrap());
            }
            Err(err) => {
                tracing::info!(message = "parse_erc721_params", err = ?err);
            }
        },
        None => match decode::decode_erc721_event_data(log.data.to_vec().as_slice()) {
            Ok((from, to, token_id)) => {
                transfer_model.from_address_hash = from.as_bytes().to_vec();
                transfer_model.to_address_hash = to.as_bytes().to_vec();
                transfer_model.token_id =
                    Some(BigDecimal::from_str(token_id.to_string().as_str()).unwrap());
            }
            Err(err) => {
                tracing::info!(message = "parse_erc721_params", err = ?err);
            }
        },
    }

    (token, transfer_model)
}

fn parse_erc1155_params(log: &Log) -> (TokenModel, TokenTransferModel) {
    let topics = decode_topics(log);

    let (mut token, mut transfer_model) = defualt_model(log);
    transfer_model.from_address_hash = topics
        .third_topic
        .map_or(consts::ZERO_ADDRESS.as_bytes().to_vec(), |topic| {
            H160::from(topic).as_bytes().to_vec()
        });
    transfer_model.to_address_hash = topics
        .fourth_topic
        .map_or(consts::ZERO_ADDRESS.as_bytes().to_vec(), |topic| {
            H160::from(topic).as_bytes().to_vec()
        });
    token.r#type = consts::ERC1155.to_string();

    if let Some(first_topic) = topics.first_topic {
        let first_topic = format!("0x{}", hex::encode(first_topic.as_bytes()));
        let first_str = first_topic.as_str();
        if first_str == consts::ERC1155_BATCH_TRANSFER_SIGNATURE {
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

fn defualt_model(log: &Log) -> (TokenModel, TokenTransferModel) {
    let transfer_model = TokenTransferModel {
        transaction_hash: log
            .transaction_hash
            .map_or(vec![], |hash| hash.as_bytes().to_vec()),
        log_index: log.log_index.map_or(0, |idx| idx.as_u32() as i32),
        from_address_hash: consts::ZERO_ADDRESS.as_bytes().to_vec(),
        to_address_hash: consts::ZERO_ADDRESS.as_bytes().to_vec(),
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
        r#type: consts::ERC20.to_string(),
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

    (token, transfer_model)
}
