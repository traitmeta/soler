use crate::common::err::ScannerError;
use crate::evms::eth::EthCli;
use anyhow::bail;
use chrono::{NaiveDateTime, Utc};
use entities::{
    addresses::Model as AddressModel, blocks::Model as BlockModel,
    internal_transactions::Model as InnerTransactionModel, logs::Model as LogModel,
    token_transfers::Model as TokenTransferModel, tokens::Model as TokenModel,
    transactions::Model as TransactionModel, withdrawals::Model as WithdrawModel,
};
use ethers::types::{Block, Trace, Transaction, TransactionReceipt, TxHash, H256, U64};
use repo::dal::{
    address::Mutation as AddressMutation,
    block::{Mutation as BlockMutation, Query as BlockQuery},
    event::Mutation as EventMutation,
    internal_transaction::Mutation as InnerTransactionMutation,
    token::Mutation as TokenMutation,
    token_transfer::Mutation as TokenTransferMutation,
    transaction::Mutation as TransactionMutation,
    withdrawal::Mutation as WithdrawalMutation,
};
use sea_orm::{
    prelude::{BigDecimal, Decimal},
    DatabaseConnection, TransactionTrait,
};
use std::time::Duration;
use std::{collections::HashMap, str::FromStr};
use tokio::time::interval;

use super::event::handle_block_event;
use super::internal_transaction::{classify_txs, handler_inner_transaction};
use super::token::handle_token_from_receipts;
use super::{address::process_block_addresses, withdrawal::withdrawals_process};

#[derive(Default)]
struct HandlerModels {
    transactions: Vec<TransactionModel>,
    events: Vec<LogModel>,
    inner_tx: Vec<InnerTransactionModel>,
    addresses: Vec<AddressModel>,
    tokens: Vec<TokenModel>,
    token_transfers: Vec<TokenTransferModel>,
    withdraws: Vec<WithdrawModel>,
}
pub struct EthHandler {
    cli: EthCli,
    conn: DatabaseConnection,
}

impl EthHandler {
    pub fn new(cli: EthCli, conn: DatabaseConnection) -> Self {
        Self { cli, conn }
    }

    pub async fn init_block(&self) {
        if let Some(block) = BlockQuery::select_latest(&self.conn).await.unwrap() {
            if block.number != 0 {
                return;
            }
        }

        let latest_block_number = self.cli.get_block_number().await;
        let latest_block = self.cli.get_block(latest_block_number).await;
        let block = self.convert_block_to_model(&latest_block);

        BlockMutation::create(&self.conn, &block).await.unwrap();
    }

    fn convert_block_to_model(&self, block: &Block<TxHash>) -> BlockModel {
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
            timestamp: NaiveDateTime::from_timestamp_opt(block.timestamp.as_u64() as i64, 0)
                .unwrap(),
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

    pub async fn sync_task(&self) {
        let mut interval = interval(Duration::from_secs(3));
        loop {
            interval.tick().await;

            let latest_block_number = self.cli.get_block_number().await;
            if let Some(latest_block) = BlockQuery::select_latest(&self.conn).await.unwrap() {
                if latest_block.number > latest_block_number as i64 {
                    tracing::info!(
                        "latestBlock.LatestBlockHeight: {} greater than latestBlockNumber: {}",
                        latest_block.number,
                        latest_block_number
                    );
                    continue;
                }
                let current_block = self
                    .cli
                    .get_block_with_tx(latest_block.number as u64 + 1)
                    .await;

                tracing::info!(
                    "get currentBlock blockNumber: {}, blockHash: {:#032x}, hash size: {}",
                    current_block.number.unwrap(),
                    current_block.hash.unwrap(),
                    current_block.hash.unwrap().as_bytes().to_vec().len(),
                );

                let block_traces = self.cli.trace_block(latest_block.number as u64 + 1).await;

                self.handle_block(&current_block, &block_traces)
                    .await
                    .unwrap();
            }
        }
    }

    

    async fn sync_to_db(
        &self,
        block: &BlockModel,
        handle_models: HandlerModels,
    ) -> anyhow::Result<()> {
        let txn = self.conn.begin().await?;

        match BlockMutation::create(&txn, block).await {
            Ok(_) => {}
            Err(e) => {
                txn.rollback().await?;
                bail!(ScannerError::Create {
                    src: "create block".to_string(),
                    err: e
                });
            }
        }
        if !handle_models.transactions.is_empty() {
            match TransactionMutation::create(&txn, &handle_models.transactions).await {
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

        if !handle_models.events.is_empty() {
            match EventMutation::create(&txn, &handle_models.events).await {
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

        if !handle_models.inner_tx.is_empty() {
            match InnerTransactionMutation::create(&txn, &handle_models.inner_tx).await {
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

        if !handle_models.addresses.is_empty() {
            match AddressMutation::save(&txn, &handle_models.addresses).await {
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

        if !handle_models.tokens.is_empty() {
            match TokenMutation::save(&txn, &handle_models.tokens).await {
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

        if !handle_models.token_transfers.is_empty() {
            match TokenTransferMutation::create(&txn, &handle_models.token_transfers).await {
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

        if !handle_models.withdraws.is_empty() {
            match WithdrawalMutation::create(&txn, &handle_models.withdraws).await {
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

        txn.commit().await?;

        Ok(())
    }
}

async fn handle_transactions(
    block: &Block<Transaction>,
    recipt_map: &HashMap<H256, TransactionReceipt>,
    trace_map: &HashMap<H256, Vec<(Trace, i32)>>,
) -> anyhow::Result<Vec<TransactionModel>> {
    let mut transactions = Vec::new();
    for tx in block.transactions.iter() {
        let recipt = recipt_map.get(&tx.hash);
        let traces = trace_map.get(&tx.hash);
        let transaction = process_transaction(tx, &block.number, recipt, traces).await?;
        transactions.push(transaction);
    }

    Ok(transactions)
}

async fn process_transaction(
    tx: &Transaction,
    block_number: &Option<U64>,
    receipt: Option<&TransactionReceipt>,
    traces: Option<&Vec<(Trace, i32)>>,
) -> anyhow::Result<TransactionModel> {
    // tracing::debug!("hand transaction, tx: {:?}", tx);
    tracing::info!("hand transaction, txHash: {:#032x}", tx.hash);

    let mut transaction = TransactionModel {
        block_number: block_number.map(|number| number.as_u64() as i32),
        hash: tx.hash.as_bytes().to_vec(),
        value: match BigDecimal::from_str(tx.value.to_string().as_str()) {
            Ok(dec) => dec,
            Err(err) => bail!(ScannerError::NewDecimal {
                src: "Pocess transaction value".to_string(),
                err: err.to_string()
            }),
        },
        status: receipt
            .as_ref()
            .map(|r| r.status)
            .and_then(|status| status.map(|s| s.as_u64() as i32)),
        cumulative_gas_used: receipt
            .as_ref()
            .map(|r| r.cumulative_gas_used)
            .map(|c| Decimal::from_i128_with_scale(c.as_usize() as i128, 0)),
        error: None,
        gas: Decimal::from_i128_with_scale(tx.gas.as_usize() as i128, 0),
        gas_price: tx
            .gas_price
            .map(|price| Decimal::from_i128_with_scale(price.as_usize() as i128, 0)),
        gas_used: receipt.as_ref().map(|r| r.gas_used).and_then(|gas_used| {
            gas_used.map(|used| Decimal::from_i128_with_scale(used.as_usize() as i128, 0))
        }),
        index: tx.transaction_index.map(|index| index.as_u64() as i32),
        input: tx.input.to_vec(),
        nonce: tx.nonce.as_u64() as i32,
        r: tx.r.to_string().into_bytes(),
        s: tx.s.to_string().into_bytes(),
        v: Decimal::new(tx.v.as_u32() as i64, 0),
        inserted_at: Utc::now().naive_utc(),
        updated_at: Utc::now().naive_utc(),
        block_hash: tx.block_hash.map(|hash| hash.as_bytes().to_vec()),
        from_address_hash: tx.from.as_bytes().to_vec(),
        to_address_hash: tx.to.map(|to| to.as_bytes().to_vec()),
        created_contract_address_hash: None,
        created_contract_code_indexed_at: None,
        earliest_processing_start: None,
        old_block_hash: None,
        revert_reason: None,
        max_priority_fee_per_gas: tx
            .max_priority_fee_per_gas
            .map(|fee| Decimal::from_i128_with_scale(fee.as_usize() as i128, 0)),

        max_fee_per_gas: tx
            .max_fee_per_gas
            .map(|fee| Decimal::from_i128_with_scale(fee.as_usize() as i128, 0)),

        r#type: receipt
            .as_ref()
            .map(|r| r.transaction_type)
            .and_then(|op_t| op_t.map(|t| t.as_u64() as i32)),

        has_error_in_internal_txs: None,
    };

    match &receipt {
        Some(receipt) => {
            if tx.to.is_none() {
                // let to_address = ethers::utils::get_contract_address(tx.from, tx.nonce).to_string();
                if let Some(contract_address) = receipt.contract_address {
                    transaction.created_contract_address_hash =
                        Some(contract_address.as_bytes().to_vec());
                }
                if let Some(to) = receipt.to {
                    transaction.created_contract_address_hash = Some(to.as_bytes().to_vec());
                }
                transaction.created_contract_code_indexed_at = Some(Utc::now().naive_utc())
            }

            if let Some(status) = receipt.status {
                if status.is_zero() {
                    // This is inner transaction
                    if let Some(trace_list) = traces {
                        for (trace, _) in trace_list.iter() {
                            transaction.error = trace.error.clone().map(|e| e.to_string());
                            transaction.revert_reason = trace.result.clone().map(|result| {
                                serde_json::to_string(&result)
                                    .unwrap_or(String::from("Error serializing value"))
                            });
                        }
                    }
                }
            }
        }
        None => {}
    }

    Ok(transaction)
}
