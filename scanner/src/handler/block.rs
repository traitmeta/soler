use entities::scanner_height::Model as ScannerBlockModel;
use entities::{
    blocks::Model as BlockModel, logs::Model as LogModel, transactions::Model as TransactionModel,
};
use ethers::types::{Block, Transaction, TransactionReceipt, TxHash, U64};
use sea_orm::{prelude::Decimal, DbConn};
use sea_orm::{DatabaseConnection, TransactionTrait};

use crate::evms::eth::EthCli;
use crate::repo::height::{Mutation, Query};
use chrono::{NaiveDateTime, Utc};
use repo::dal::block::{Mutation as BlockMutation, Query as BlockQuery};
use repo::dal::event::Mutation as EventMutation;
use repo::dal::transaction::Mutation as TransactionMutation;
use std::time::Duration;
use tokio::time::interval;

pub struct EthHandler {
    cli: EthCli,
    conn: DatabaseConnection,
}
impl EthHandler {
    pub fn new(cli: EthCli, conn: DatabaseConnection) -> Self {
        Self { cli, conn }
    }

    pub async fn init_block(&self) {
        if let Some(db_heigest) = BlockQuery::select_latest(&self.conn).await.unwrap() {
            if db_heigest.number == 0 {
                let latest_block_number = self.cli.get_block_number().await;
                let latest_block = self.cli.get_block(latest_block_number).await;
                let block = self.convert_block_to_model(&latest_block);

                BlockMutation::create(&self.conn, &block).await.unwrap();
            };
        };
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
                Some(hash) => hash.to_string().into_bytes(),
                None => vec![],
            },
            miner_hash: match block.author {
                Some(hash) => hash.to_string().into_bytes(),
                None => vec![],
            },
            nonce: match block.nonce {
                Some(nonce) => nonce.to_string().into_bytes(),
                None => vec![],
            },
            number: match block.number {
                Some(number) => number.as_u64() as i64,
                None => 0,
            },
            parent_hash: block.parent_hash.to_string().into_bytes(),
            size: match block.size {
                Some(size) => Some(size.as_u32() as i32),
                None => None,
            },
            timestamp: NaiveDateTime::from_timestamp_millis(block.timestamp.as_u64() as i64)
                .unwrap(),
            base_fee_per_gas: match block.base_fee_per_gas {
                Some(base_fee_per_gas) => Some(Decimal::from_i128_with_scale(
                    base_fee_per_gas.as_u128() as i128,
                    0,
                )),
                None => None,
            },
            consensus: true,
            total_difficulty: match block.total_difficulty {
                Some(total_difficulty) => Some(Decimal::from_i128_with_scale(
                    total_difficulty.as_u128() as i128,
                    0,
                )),
                None => None,
            },
            inserted_at: Utc::now().naive_utc(),
            updated_at: Utc::now().naive_utc(),
            refetch_needed: Some(false),
            is_empty: Some(false),
        };

        block
    }

    pub async fn sync_task(&self) {
        let mut interval = interval(Duration::from_secs(1));
        loop {
            interval.tick().await;

            let latest_block_number = self.cli.get_block_number().await;
            if let Some(latest_block) = BlockQuery::select_latest(&self.conn).await.unwrap() {
                if latest_block.number > latest_block_number as i64{
                    tracing::info!(
                        "latestBlock.LatestBlockHeight: {} greater than latestBlockNumber: {}",
                        latest_block.number,
                        latest_block_number
                    );
                    continue;
                }
                let current_block = self.cli.get_block_with_tx(latest_block.number as u64 + 1).await;

                tracing::info!(
                    "get currentBlock blockNumber: {}, blockHash: {}",
                    current_block.number.unwrap(),
                    current_block.hash.unwrap()
                );

                self.handle_block(&current_block).await.unwrap();
            }
        }
    }

    async fn handle_block(
        &self,
        block: &Block<Transaction>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let block_model = BlockModel {
            difficulty: Some(Decimal::from_i128_with_scale(
                block.difficulty.as_u128() as i128,
                0,
            )),
            gas_limit: Decimal::from_i128_with_scale(block.gas_limit.as_u128() as i128, 0),
            gas_used: Decimal::from_i128_with_scale(block.gas_used.as_u128() as i128, 0),
            hash: match block.hash {
                Some(hash) => hash.to_string().into_bytes(),
                None => vec![],
            },
            miner_hash: match block.author {
                Some(hash) => hash.to_string().into_bytes(),
                None => vec![],
            },
            nonce: match block.nonce {
                Some(nonce) => nonce.to_string().into_bytes(),
                None => vec![],
            },
            number: match block.number {
                Some(number) => number.as_u64() as i64,
                None => 0,
            },
            parent_hash: block.parent_hash.to_string().into_bytes(),
            size: match block.size {
                Some(size) => Some(size.as_u32() as i32),
                None => None,
            },
            timestamp: NaiveDateTime::from_timestamp_millis(block.timestamp.as_u64() as i64)
                .unwrap(),
            base_fee_per_gas: match block.base_fee_per_gas {
                Some(base_fee_per_gas) => Some(Decimal::from_i128_with_scale(
                    base_fee_per_gas.as_u128() as i128,
                    0,
                )),
                None => None,
            },
            consensus: true,
            total_difficulty: match block.total_difficulty {
                Some(total_difficulty) => Some(Decimal::from_i128_with_scale(
                    total_difficulty.as_u128() as i128,
                    0,
                )),
                None => None,
            },
            inserted_at: Utc::now().naive_utc(),
            updated_at: Utc::now().naive_utc(),
            refetch_needed: Some(false),
            is_empty: Some(block.transactions.len() == 0), // transaction count is zero
        };
        let recipts = self.cli.get_block_receipt(block_model.number as u64).await;
        let transactions = self.handle_transactions(&block, &recipts).await?;
        let events = Self::handle_block_event(&recipts);
        self.sync_to_db(&block_model, &transactions, &events)
            .await?;

        Ok(())
    }

    async fn sync_to_db(
        &self,
        block: &BlockModel,
        transactions: &Vec<TransactionModel>,
        events: &Vec<LogModel>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let txn = self.conn.begin().await?;

        match BlockMutation::create(&txn, &block).await {
            Ok(_) => {}
            Err(e) => {
                txn.rollback().await?;
                return Err(Box::new(e));
            }
        }
        
        match TransactionMutation::create(&txn, transactions).await {
            Ok(_) => {}
            Err(e) => {
                txn.rollback().await?;
                return Err(Box::new(e));
            }
        }

        match EventMutation::create(&txn, events).await {
            Ok(_) => {}
            Err(e) => {
                txn.rollback().await?;
                return Err(Box::new(e));
            }
        }

        txn.commit().await?;

        Ok(())
    }

    async fn handle_transactions(
        &self,
        block: &Block<Transaction>,
        recipts: &Vec<TransactionReceipt>,
    ) -> Result<Vec<TransactionModel>, Box<dyn std::error::Error>> {
        let mut transactions = Vec::new();
        for tx in block.transactions.iter() {
            let mut tx_receipt = None;
            for receipt in recipts.iter() {
                if receipt.transaction_hash.eq(&tx.hash) {
                    tx_receipt = Some(receipt);
                }
            }

            let transaction = self
                .process_transaction(tx, &block.number, &tx_receipt)
                .await
                .unwrap();
            transactions.push(transaction);
        }

        Ok(transactions)
    }

    async fn process_transaction(
        &self,
        tx: &Transaction,
        block_number: &Option<U64>,
        receipt: &Option<&TransactionReceipt>,
    ) -> Result<TransactionModel, Box<dyn std::error::Error>> {
        tracing::info!("hand transaction, txHash: {}", tx.hash);

        // TODO fulfill transaction err info
        let mut transaction = TransactionModel {
            block_number: match block_number {
                Some(block_number) => Some(block_number.as_u64() as i32),
                None => None,
            },
            hash: tx.hash.to_string().into_bytes(),
            value: Decimal::from_i128_with_scale(tx.value.as_u128() as i128, 0),
            status: match receipt {
                Some(receipt) => match receipt.status {
                    Some(status) => Some(status.as_u64() as i32),
                    None => None,
                },
                None => None,
            },
            cumulative_gas_used: match receipt {
                Some(r) => Some(Decimal::from_i128_with_scale(
                    r.cumulative_gas_used.as_u128() as i128,
                    0,
                )),
                None => None,
            },
            error: None,
            gas: Decimal::from_i128_with_scale(tx.gas.as_u128() as i128, 0),
            gas_price: match tx.gas_price {
                Some(gas_price) => Some(Decimal::from_i128_with_scale(
                    gas_price.as_u128() as i128,
                    0,
                )),
                None => None,
            },
            gas_used: match receipt {
                Some(r) => match r.gas_used {
                    Some(gas_used) => {
                        Some(Decimal::from_i128_with_scale(gas_used.as_u128() as i128, 0))
                    }
                    None => None,
                },
                None => None,
            },
            index: match tx.transaction_index {
                Some(index) => Some(index.as_u64() as i32),
                None => None,
            },
            input: tx.input.to_vec(),
            nonce: tx.nonce.as_u64() as i32,
            r: Decimal::from_i128_with_scale(tx.r.as_u128() as i128, 0),
            s: Decimal::from_i128_with_scale(tx.s.as_u128() as i128, 0),
            v: Decimal::from_i128_with_scale(tx.v.as_u32() as i128, 0),
            inserted_at: Utc::now().naive_utc(),
            updated_at: Utc::now().naive_utc(),
            block_hash: match tx.block_hash {
                Some(hash) => Some(hash.to_string().into_bytes()),
                None => None,
            },
            from_address_hash: tx.from.to_string().into_bytes(),
            to_address_hash: None,
            created_contract_address_hash: None,
            created_contract_code_indexed_at: None,
            earliest_processing_start: None,
            old_block_hash: None,
            revert_reason: None,
            max_priority_fee_per_gas: match tx.max_priority_fee_per_gas {
                Some(max_priority_fee_per_gas) => Some(Decimal::from_i128_with_scale(
                    max_priority_fee_per_gas.as_u128() as i128,
                    0,
                )),
                None => None,
            },
            max_fee_per_gas: match tx.max_fee_per_gas {
                Some(max_fee_per_gas) => Some(Decimal::from_i128_with_scale(
                    max_fee_per_gas.as_u128() as i128,
                    0,
                )),
                None => None,
            },
            r#type: match receipt {
                Some(r) => match r.transaction_type {
                    Some(transaction_type) => Some(transaction_type.as_u64() as i32),
                    None => None,
                },
                None => None,
            },
            has_error_in_internal_txs: None,
        };

        if tx.to.is_none() {
            tracing::info!(
                "Contract creation found, Sender: {}, TxHash: {}",
                tx.from,
                tx.hash
            );
            let to_address = ethers::utils::get_contract_address(tx.from, tx.nonce).to_string();
            transaction.to_address_hash = Some(to_address.into_bytes());
        }

        match receipt {
            Some(receipt) => match receipt.status {
                Some(status) => {
                    if status.as_u64() == 1 {
                        ()
                    }

                    let traces = self.cli.trace_transaction(receipt.transaction_hash).await;
                    for trace in traces.iter() {
                        transaction.error = match &trace.error {
                            Some(error) => Some(error.clone()),
                            None => None,
                        };
                        transaction.revert_reason = match &trace.result {
                            Some(result) => Some(serde_json::to_string(result).unwrap()),
                            None => None,
                        }
                    }
                }
                None => (),
            },
            None => {}
        }

        Ok(transaction)
    }

    fn handle_block_event(receipts: &Vec<TransactionReceipt>) -> Vec<LogModel> {
        let mut events = Vec::new();
        for receipt in receipts.iter() {
            for log in receipt.logs.iter() {
                let mut event = LogModel {
                    data: log.data.to_vec(),
                    index: match log.log_index {
                        Some(index) => index.as_u64() as i32,
                        None => 0,
                    },
                    r#type: log.log_type.clone(),
                    first_topic: None,
                    second_topic: None,
                    third_topic: None,
                    fourth_topic: None,
                    address_hash: Some(log.address.to_string().into_bytes()),
                    transaction_hash: match log.transaction_hash {
                        Some(hash) => hash.to_string().into_bytes(),
                        None => vec![],
                    },
                    block_hash: match log.block_hash {
                        Some(hash) => hash.to_string().into_bytes(),
                        None => vec![],
                    },
                    block_number: match log.block_number {
                        Some(number) => Some(number.as_u64() as i32),
                        None => None,
                    },
                    inserted_at: Utc::now().naive_utc(),
                    updated_at: Utc::now().naive_utc(),
                };

                for (i, topic) in log.topics.iter().enumerate() {
                    match i {
                        0 => event.first_topic = Some(topic.to_string()),
                        1 => event.second_topic = Some(topic.to_string()),
                        2 => event.third_topic = Some(topic.to_string()),
                        3 => event.fourth_topic = Some(topic.to_string()),
                        _ => (),
                    }
                }
                events.push(event);
            }
        }

        events
    }
}

pub async fn current_height(conn: &DbConn, task_name: &str, chain_name: &str) -> Option<u64> {
    let current_model = Query::select_one_by_task_name(conn, task_name)
        .await
        .unwrap();
    match current_model {
        Some(current) => return Some(current.height),
        None => {
            tracing::debug!("not found {}", task_name);
            let insert_data = ScannerBlockModel {
                id: 0,
                task_name: task_name.to_owned(),
                chain_name: chain_name.to_owned(),
                height: 1,
                created_at: None,
                updated_at: None,
            };
            let result = Mutation::create_scanner_height(conn, insert_data)
                .await
                .unwrap_or_else(|_| panic!("insert {} to scanner height table err", task_name));
            tracing::debug!("insert {} return :{:?}", task_name, result);
        }
    }
    None
}
