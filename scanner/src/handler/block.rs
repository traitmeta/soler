use entities::scanner_height::Model as ScannerBlockModel;
use entities::{
    blocks::ActiveModel as BlockModel, logs::ActiveModel as LogModel, transactions::Model as TransactionModel,
};
use futures::AsyncReadExt;
use sea_orm::{DbConn,ActiveValue,prelude::Decimal};

use crate::evms::eth::EthCli;
use crate::repo::height::{Mutation, Query};
use std::time::Duration;
use tokio::time::interval;

use chrono::{NaiveDate, NaiveDateTime};
use web3::types::{Block, BlockNumber};

use config::{chain::Chain, db::DB};

pub struct EthHandler {
    cli: EthCli,
    db: DB,
}
impl EthHandler {
    pub fn new(cli: EthCli, db: DB) -> Self {
        Self { cli, db }
    }

    pub async fn init_block(&self) {
        // let block_count = self.db.block.count().await.unwrap();
        let block_count = 0;
        if block_count == 0 {
            let latest_block_number = self.cli.get_block_number().await;
            let latest_block = self.cli.get_block(latest_block_number).await;
            // TODO use model when dal use active Model
            let block = BlockModel {
                difficulty: ActiveValue::Set(Some(Decimal::from_i128_with_scale(
                    latest_block.difficulty.as_u128() as i128,
                    0,
                ))),
                gas_limit: ActiveValue::Set(Decimal::from_i128_with_scale(
                    latest_block.gas_limit.as_u128() as i128,
                    0,
                )),
                gas_used: ActiveValue::Set(Decimal::from_i128_with_scale(latest_block.gas_used.as_u128() as i128, 0)),
                hash: ActiveValue::Set(match latest_block.hash{
                    Some(hash) => hash.to_string().into_bytes(),
                    None => vec![],
                }),
                miner_hash:  ActiveValue::Set(latest_block.author.to_string().into_bytes()),
                nonce: ActiveValue::Set(match latest_block.nonce{
                    Some(nonce) => nonce.to_string().into_bytes(),
                    None => vec![],
                }),
                number: ActiveValue::Set(match latest_block.number{
                    Some(number) => number.as_u64(),
                    None => 0,
                }),
                parent_hash: ActiveValue::Set(latest_block.parent_hash.to_string().into_bytes()),
                size: ActiveValue::Set(match latest_block.size {
                    Some(size) => Some(size.as_u32() as i32),
                    None => None,
                }),
                timestamp: ActiveValue::Set(NaiveDateTime::from_timestamp_millis(latest_block.timestamp.as_u64() as i64).unwrap()),
                base_fee_per_gas: ActiveValue::Set(match latest_block.base_fee_per_gas {
                    Some(base_fee_per_gas) => Some(Decimal::from_i128_with_scale(
                        base_fee_per_gas.as_u128() as i128,
                        0,
                    )),
                    None => None,
                }),
                ..Default::default()
            };


            self.db.block.insert(&block).await.unwrap();
        }
    }

    // TODO
    pub async fn sync_task(&self, config: Chain) {
        let mut interval = interval(Duration::from_secs(1));
        loop {
            interval.tick().await;

            let latest_block_number = self.cli.get_block_number().await;
            let latest_block = self.db.block.get_latest().await.unwrap();

            if latest_block.latest_block_height > latest_block_number {
                log::info!(
                    "latestBlock.LatestBlockHeight: {} greater than latestBlockNumber: {}",
                    latest_block.latest_block_height,
                    latest_block_number
                );
                continue;
            }
            let current_block = self
                .cli
                .get_block_with_tx(latest_block.latest_block_height)
                .await;

            log::info!(
                "get currentBlock blockNumber: {}, blockHash: {}",
                current_block.number.unwrap(),
                current_block.hash.unwrap()
            );

            Self::handle_block(&db, &config, &current_block)
                .await
                .unwrap();
        }
    }

    async fn handle_block(
        db: &DB,
        config: &Config,
        block: &Block<Transaction>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let block_model = BlockModel {
            hash: ActiveValue::Set(match latest_block.hash{
                Some(hash) => hash.to_string().into_bytes(),
                None => vec![],
            }),
            number: ActiveValue::Set(match latest_block.number{
                Some(number) => number.as_u64(),
                None => 0,
            }),
            parent_hash: ActiveValue::Set(latest_block.parent_hash.to_string().into_bytes()),
            ..Default::default()
        };

        let (events, transactions) = Self::handle_transactions(&Self, &block).await?;

        Self::sync_to_db(db, &block_model, &transactions, &events).await?;

        Ok(())
    }

    async fn sync_to_db(
        db: &DB,
        block: &BlockModel,
        transactions: &Vec<TransactionModel>,
        events: &Vec<LogModel>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let tx = db.transaction().await?;

        db.block.insert(&tx, block).await?;
        db.transaction.inserts(&tx, transactions).await?;
        db.event.inserts(&tx, events).await?;

        tx.commit().await?;

        Ok(())
    }

    async fn handle_transactions(
        &self,
        block: &Block<Transaction>,
    ) -> Result<(Vec<LogModel>, Vec<TransactionModel>), Box<dyn std::error::Error>> {
        let mut events = Vec::new();
        let mut transactions = Vec::new();

        for tx in block.transactions.iter() {
            let receipt = self.cli.get_transaction_receipt(&tx.hash.unwrap()).await;
            for log in receipt.logs.iter() {
                let status = receipt.status.unwrap();
                let event = Self::handle_transaction_event(log, status);
                match event {
                    Ok(e) => events.push(e),
                    Err(e) => {
                        log::error!("{:?}", e);
                    }
                }
                events.push(event);
            }

            let transaction =
                Self::process_transaction(tx, &block.number.unwrap(), receipt.status)?;
            transactions.push(transaction);
        }

        Ok((events, transactions))
    }

    fn process_transaction(
        tx: &web3::types::Transaction,
        block_number: &U256,
        status: U256,
    ) -> Result<TransactionModel, Box<dyn std::error::Error>> {
        let from = tx
            .clone()
            .from
            .ok_or_else(|| format!("Failed to read the sender address: {:?}", tx.hash))?;

        log::info!("hand transaction, txHash: {}", tx.hash.unwrap());

        let mut transaction = TransactionModel {
            block_number: block_number.as_u64(),
            tx_hash: tx.hash.unwrap().to_string(),
            from: from.to_string(),
            value: tx.value.to_string(),
            status: status.as_u64(),
            input_data: hex::encode(&tx.input),
            ..Default::default()
        };

        if tx.to.is_none() {
            log::info!(
                "Contract creation found, Sender: {}, TxHash: {}",
                transaction.from,
                transaction.tx_hash
            );
            let to_address = crypto::create_address(&from, tx.nonce).to_string();
            transaction.contract = Some(to_address);
        } else {
            let is_contract = is_contract_address(&tx.to.unwrap()).await?;
            if is_contract {
                transaction.contract = tx.to.clone().map(|address| address.to_string());
            } else {
                transaction.to = tx.to.clone().map(|address| address.to_string());
            }
        }

        Ok(transaction)
    }

    fn handle_transaction_event(
        log: &web3::types::Log,
        status: U64,
    ) -> Result<LogModel, Box<dyn std::error::Error>> {
        log::info!(
            "ProcessTransactionEvent, address: {}, data: {}",
            log.address,
            hex::encode(&log.data)
        );

        let mut event = LogModel {
            data: ActiveValue::Set(log.data.to_string()),
            index: ActiveValue::Set(log.log_index),
            r#type: ActiveValue::Set(log.log_type),
            first_topic: ActiveValue::Set(None),
            second_topic: ActiveValue::Set(None),
            third_topic: ActiveValue::Set(None),
            fourth_topic: ActiveValue::Set(None),
            address_hash: ActiveValue::Set(log.address.to_string()),
            transaction_hash: ActiveValue::Set(log.transaction_hash),
            block_hash: ActiveValue::Set(log.block_hash),
            block_number: ActiveValue::Set(log.block_number),
            ..Default::default()
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

        Ok(event)
    }

    async fn is_contract_address(address: &str) -> Result<bool, Box<dyn std::error::Error>> {
        let addr = address.parse()?;
        let code = cli.code_at(&addr, None).await?;

        Ok(!code.is_empty())
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
