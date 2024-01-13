use std::{collections::HashMap, sync::Arc};

use anyhow::bail;
use chrono::{NaiveDateTime, Utc};
use entities::{
    address_current_token_balances::Model as CurrentTokenBalanceModel,
    address_token_balances::Model as AddressTokenBalanceModel, addresses::Model as AddressModel,
    blocks::Model as BlockModel, internal_transactions::Model as InnerTransactionModel,
    logs::Model as LogModel, token_transfers::Model as TokenTransferModel,
    tokens::Model as TokenModel, transactions::Model as TransactionModel,
    withdrawals::Model as WithdrawModel,
};
use ethers::types::{Block, Trace, Transaction, TransactionReceipt, TxHash};
use repo::dal::{
    address::Mutation as AddressMutation,
    block::{Mutation as BlockMutation, Query as BlockQuery},
    current_token_balance::Mutation as CurrentTokenMutation,
    event::Mutation as EventMutation,
    internal_transaction::Mutation as InnerTransactionMutation,
    token::Mutation as TokenMutation,
    token_balance::Mutation as TokenBalanceMutation,
    token_transfer::Mutation as TokenTransferMutation,
    transaction::Mutation as TransactionMutation,
    withdrawal::Mutation as WithdrawalMutation,
};
use sea_orm::{prelude::Decimal, DatabaseConnection, DbConn, TransactionTrait};

use super::internal_transaction::{classify_txs, handler_inner_transaction};
use super::token::handle_token_from_receipts;
use super::{address::process_block_addresses, withdrawal::withdrawals_process};
use super::{event::handle_block_event, transaction::handle_transactions};
use crate::common::err::ScannerError;
use crate::evms::eth::EthCli;

pub struct HandlerModels {
    block: BlockModel,
    datas: DataModels,
}

#[derive(Default)]
pub struct DataModels {
    transactions: Vec<TransactionModel>,
    events: Vec<LogModel>,
    inner_tx: Vec<InnerTransactionModel>,
    addresses: Vec<AddressModel>,
    tokens: Vec<TokenModel>,
    token_transfers: Vec<TokenTransferModel>,
    withdraws: Vec<WithdrawModel>,
    address_token_balance: Vec<AddressTokenBalanceModel>,
    current_token_balance: Vec<CurrentTokenBalanceModel>,
}

pub async fn init_block(cli: Arc<EthCli>, conn: Arc<DatabaseConnection>) {
    if let Some(block) = BlockQuery::select_latest(conn.as_ref()).await.unwrap() {
        if block.number != 0 {
            return;
        }
    }

    let latest_block_number = cli.get_block_number().await;
    let latest_block = cli.get_block(latest_block_number).await;
    let block = convert_block_to_model(&latest_block);

    BlockMutation::create(conn.as_ref(), &block).await.unwrap();
}

fn convert_block_to_model(block: &Block<TxHash>) -> BlockModel {
    let block = BlockModel {
        difficulty: Some(Decimal::from_i128_with_scale(
            block.difficulty.as_u128() as i128,
            0,
        )),
        gas_limit: Decimal::from_i128_with_scale(block.gas_limit.as_u128() as i128, 0),
        gas_used: Decimal::from_i128_with_scale(block.gas_used.as_u128() as i128, 0),
        hash: match block.hash {
            Some(hash) => hash.as_bytes().to_vec(),
            None => vec![],
        },
        miner_hash: match block.author {
            Some(hash) => hash.as_bytes().to_vec(),
            None => vec![],
        },
        nonce: match block.nonce {
            Some(nonce) => nonce.as_bytes().to_vec(),
            None => vec![],
        },
        number: match block.number {
            Some(number) => number.as_u64() as i64,
            None => 0,
        },
        parent_hash: block.parent_hash.as_bytes().to_vec(),
        size: block.size.map(|size| size.as_u32() as i32),
        timestamp: NaiveDateTime::from_timestamp_opt(block.timestamp.as_u64() as i64, 0).unwrap(),
        base_fee_per_gas: block.base_fee_per_gas.map(|base_fee_per_gas| {
            Decimal::from_i128_with_scale(base_fee_per_gas.as_u128() as i128, 0)
        }),
        consensus: true,
        total_difficulty: block.total_difficulty.map(|total_difficulty| {
            Decimal::from_i128_with_scale(total_difficulty.as_u128() as i128, 0)
        }),
        inserted_at: Utc::now().naive_utc(),
        updated_at: Utc::now().naive_utc(),
        refetch_needed: Some(false),
        is_empty: Some(false),
    };

    block
}

pub async fn handle_block(
    block: &Block<Transaction>,
    traces: &[Trace],
    recipts: &[TransactionReceipt],
) -> anyhow::Result<HandlerModels> {
    let data_model = parse_block(block, traces, recipts).await?;
    let block_header = handle_block_header(block).await?;
    Ok(HandlerModels {
        block: block_header,
        datas: data_model,
    })
}

pub async fn handle_block_header(block: &Block<Transaction>) -> anyhow::Result<BlockModel> {
    let block_model = BlockModel {
        difficulty: Some(Decimal::from_i128_with_scale(
            block.difficulty.as_u128() as i128,
            0,
        )),
        gas_limit: Decimal::from_i128_with_scale(block.gas_limit.as_u128() as i128, 0),
        gas_used: Decimal::from_i128_with_scale(block.gas_used.as_u128() as i128, 0),
        hash: match block.hash {
            Some(hash) => hash.as_bytes().to_vec(),
            None => vec![],
        },
        miner_hash: match block.author {
            Some(hash) => hash.as_bytes().to_vec(),
            None => vec![],
        },
        nonce: match block.nonce {
            Some(nonce) => nonce.as_bytes().to_vec(),
            None => vec![],
        },
        number: match block.number {
            Some(number) => number.as_u64() as i64,
            None => 0,
        },
        parent_hash: block.parent_hash.as_bytes().to_vec(),
        size: block.size.map(|size| size.as_u32() as i32),
        timestamp: NaiveDateTime::from_timestamp_opt(block.timestamp.as_u64() as i64, 0).unwrap(),
        base_fee_per_gas: block.base_fee_per_gas.map(|base_fee_per_gas| {
            Decimal::from_i128_with_scale(base_fee_per_gas.as_u128() as i128, 0)
        }),
        consensus: true,
        total_difficulty: block.total_difficulty.map(|total_difficulty| {
            Decimal::from_i128_with_scale(total_difficulty.as_u128() as i128, 0)
        }),
        inserted_at: Utc::now().naive_utc(),
        updated_at: Utc::now().naive_utc(),
        refetch_needed: Some(false),
        is_empty: Some(block.transactions.is_empty()), // transaction count is zero
    };

    Ok(block_model)
}

pub async fn parse_block(
    block: &Block<Transaction>,
    traces: &[Trace],
    recipts: &[TransactionReceipt],
) -> anyhow::Result<DataModels> {
    let mut data_models = DataModels {
        withdraws: withdrawals_process(block.hash, block.withdrawals.clone()),
        ..Default::default()
    };

    let recipet_map = recipts
        .iter()
        .map(|r| (r.transaction_hash, r.clone()))
        .collect::<HashMap<_, _>>();

    let trace_map = classify_txs(traces);
    data_models.transactions = handle_transactions(block, &recipet_map, &trace_map).await?;
    data_models.events = handle_block_event(recipts);
    data_models.inner_tx = handler_inner_transaction(traces);
    data_models.addresses = process_block_addresses(block, &recipet_map, &trace_map);
    (
        data_models.tokens,
        data_models.token_transfers,
        data_models.address_token_balance,
        data_models.current_token_balance,
    ) = handle_token_from_receipts(recipts);

    Ok(data_models)
}

pub async fn sync_to_db(conn: &DbConn, handle_models: HandlerModels) -> anyhow::Result<()> {
    let txn = conn.begin().await?;

    match BlockMutation::create(&txn, &handle_models.block).await {
        Ok(_) => {}
        Err(e) => {
            txn.rollback().await?;
            bail!(ScannerError::Create {
                src: "create block".to_string(),
                err: e
            });
        }
    }
    if !handle_models.datas.transactions.is_empty() {
        match TransactionMutation::create(&txn, &handle_models.datas.transactions).await {
            Ok(_) => {}
            Err(e) => {
                txn.rollback().await?;
                bail!(ScannerError::Create {
                    src: "create transactions".to_string(),
                    err: e
                });
            }
        }
    }

    if !handle_models.datas.events.is_empty() {
        match EventMutation::create(&txn, &handle_models.datas.events).await {
            Ok(_) => {}
            Err(e) => {
                txn.rollback().await?;
                bail!(ScannerError::Create {
                    src: "create events".to_string(),
                    err: e
                });
            }
        }
    }

    if !handle_models.datas.inner_tx.is_empty() {
        match InnerTransactionMutation::create(&txn, &handle_models.datas.inner_tx).await {
            Ok(_) => {}
            Err(e) => {
                txn.rollback().await?;
                bail!(ScannerError::Create {
                    src: "create internal transactions".to_string(),
                    err: e
                });
            }
        }
    }

    if !handle_models.datas.addresses.is_empty() {
        match AddressMutation::save(&txn, &handle_models.datas.addresses).await {
            Ok(_) => {}
            Err(e) => {
                txn.rollback().await?;
                bail!(ScannerError::Upsert {
                    src: "save addresses".to_string(),
                    err: e
                });
            }
        }
    }

    if !handle_models.datas.tokens.is_empty() {
        match TokenMutation::save(&txn, &handle_models.datas.tokens).await {
            Ok(_) => {}
            Err(e) => {
                txn.rollback().await?;
                bail!(ScannerError::Upsert {
                    src: "create tokens".to_string(),
                    err: e
                });
            }
        }
    }

    if !handle_models.datas.token_transfers.is_empty() {
        match TokenTransferMutation::create(&txn, &handle_models.datas.token_transfers).await {
            Ok(_) => {}
            Err(e) => {
                txn.rollback().await?;
                bail!(ScannerError::Upsert {
                    src: "create token transfers".to_string(),
                    err: e
                });
            }
        }
    }

    if !handle_models.datas.withdraws.is_empty() {
        match WithdrawalMutation::create(&txn, &handle_models.datas.withdraws).await {
            Ok(_) => {}
            Err(e) => {
                txn.rollback().await?;
                bail!(ScannerError::Upsert {
                    src: "create withdraws".to_string(),
                    err: e
                });
            }
        }
    }

    if !handle_models.datas.address_token_balance.is_empty() {
        match TokenBalanceMutation::save(&txn, &handle_models.datas.address_token_balance).await {
            Ok(_) => {}
            Err(e) => {
                txn.rollback().await?;
                bail!(ScannerError::Upsert {
                    src: "create address token balance".to_string(),
                    err: e
                });
            }
        }
    }

    if !handle_models.datas.current_token_balance.is_empty() {
        match CurrentTokenMutation::save(&txn, &handle_models.datas.current_token_balance).await {
            Ok(_) => {}
            Err(e) => {
                txn.rollback().await?;
                bail!(ScannerError::Upsert {
                    src: "create address current token balance".to_string(),
                    err: e
                });
            }
        }
    }

    txn.commit().await?;

    Ok(())
}
