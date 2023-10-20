use crate::{
    common::{self, consts},
    contracts::{decode, erc20::IERC20Call},
};
use anyhow::{anyhow, Error};
use chrono::Utc;
use entities::token_transfers::Model as TokenTransferModel;
use entities::tokens::Model as TokenModel;
use ethers::types::{Log, TransactionReceipt, H160, H256, U256};
use repo::dal::token::{Mutation, Query};
use sea_orm::{
    prelude::{BigDecimal, Decimal},
    DbConn, Related,
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
            0 => topics.first_topic = Some(topic.clone()),
            1 => topics.second_topic = Some(topic.clone()),
            2 => topics.third_topic = Some(topic.clone()),
            3 => topics.fourth_topic = Some(topic.clone()),
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

pub fn token_process(logs: Vec<Log>) -> (Vec<TokenModel>, Vec<TokenTransferModel>) {
    let mut acc = (vec![], vec![]);
    for log in logs {
        acc = do_parse(&log, acc);
    }

    let (tokens, token_transfers) = sanitize_token_types(acc.0, acc.1);

    let token_transfers_filtered = token_transfers
        .into_iter()
        .filter(|transfer| {
            transfer.to_address_hash == consts::BURN_ADDRESS.as_bytes().to_vec()
                || transfer.from_address_hash == consts::BURN_ADDRESS.as_bytes().to_vec()
        })
        .collect::<Vec<TokenTransferModel>>();

    let token_contract_addresses = token_transfers_filtered
        .iter()
        .map(|transfer| transfer.token_contract_address_hash.clone())
        .collect::<Vec<Vec<u8>>>();

    let unique_token_contract_addresses = token_contract_addresses
        .iter()
        .cloned()
        .collect::<HashSet<Vec<u8>>>();

    // TokenTotalSupplyUpdater::add_tokens(unique_token_contract_addresses);
    let mut tokens_uniq = vec![];
    let mut tokens_uniq_uniq: HashSet<Vec<u8>> = HashSet::new();
    for token in tokens {
        if tokens_uniq_uniq.get(&token.contract_address_hash).is_some() {
            continue;
        } else {
            tokens_uniq_uniq.insert(token.contract_address_hash);
            tokens_uniq.push(token);
        }
    }

    let token_transfers = token_transfers_filtered.into_iter().clone().collect();

    (tokens_uniq, token_transfers)
}

fn sanitize_token_types(
    tokens: Vec<TokenModel>,
    token_transfers: Vec<TokenTransferModel>,
) -> (Vec<TokenModel>, Vec<TokenTransferModel>) {
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

fn define_token_type(token_transfers: Vec<TokenTransferModel>) -> String {
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
    mut acc: (Vec<TokenModel>, Vec<TokenTransferModel>),
) -> (Vec<TokenModel>, Vec<TokenTransferModel>) {
    let decoded_topics = decode_topics(log);
    match decoded_topics.first_topic {
        Some(first) => {
            let first_str = first.as_bytes();
            let mut kind = TokenKind::None;
            if first_str == consts::TOKEN_TRANSFER_SIGNATURE.as_bytes()
                || first_str == consts::WETH_DEPOSIT_SIGNATURE.as_bytes()
                || first_str == consts::WETH_WITHDRAWAL_SIGNATURE.as_bytes()
            {
                kind = decode_token_type(decoded_topics);
            } else if first_str == consts::ERC1155_BATCH_TRANSFER_SIGNATURE.as_bytes()
                || first_str == consts::ERC1155_SINGLE_TRANSFER_SIGNATURE.as_bytes()
            {
                kind = TokenKind::ERC1155;
            }
            let (token, token_transfer) = match kind {
                TokenKind::ERC1155 => parse_erc1155_params(log),
                TokenKind::ERC20 => parse_erc20_params(log),
                TokenKind::Erc721Data | TokenKind::Erc721Topic => parse_erc721_params(log),
                TokenKind::Erc20Weth => parse_weth_params(log),
                _ => unreachable!(),
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

    let amount = match decode::decode_erc20_event_data(log.data.to_vec().as_slice()) {
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

    let amount = match decode::decode_erc20_event_data(log.data.to_vec().as_slice()) {
        Ok(value) => {
            transfer_model.amount = Some(BigDecimal::from_str(value.to_string().as_str()).unwrap());
        }
        Err(err) => {
            tracing::info!(message = "parse_weth_params", err = ?err);
        }
    };

    if let Some(first_topic) = topics.first_topic {
        if H160::from(first_topic).as_bytes() == consts::WETH_DEPOSIT_SIGNATURE.as_bytes() {
            if let Some(second_topic) = topics.second_topic {
                transfer_model.to_address_hash = H160::from(second_topic).as_bytes().to_vec();
            }
        } else {
            if let Some(second_topic) = topics.second_topic {
                transfer_model.from_address_hash = H160::from(second_topic).as_bytes().to_vec();
            }
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

    if topics.second_topic.is_some() {
        match decode::decode_erc20_event_data(log.data.to_vec().as_slice()) {
            Ok(value) => {
                transfer_model.token_id =
                    Some(BigDecimal::from_str(value.to_string().as_str()).unwrap());
            }
            Err(err) => {
                tracing::info!(message = "parse_erc721_params", err = ?err);
            }
        };
    } else {
        match decode::decode_erc721_event_data(log.data.to_vec().as_slice()) {
            Ok((from, to, token_id)) => {
                transfer_model.from_address_hash = from.as_bytes().to_vec();
                transfer_model.to_address_hash = to.as_bytes().to_vec();
                transfer_model.token_id =
                    Some(BigDecimal::from_str(token_id.to_string().as_str()).unwrap());
            }
            Err(err) => {
                tracing::info!(message = "parse_erc721_params", err = ?err);
            }
        };
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
        if H160::from(first_topic).as_bytes() == consts::ERC1155_BATCH_TRANSFER_SIGNATURE.as_bytes()
        {
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
    let mut transfer_model = TokenTransferModel {
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

#[allow(dead_code)]
fn encode_address_hash(binary: Vec<u8>) -> String {
    format!("0x{}", hex::encode(binary))
}

#[allow(dead_code)]
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
