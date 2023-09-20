use entities::scanner_height::Model as ScannerBlockModel;
use entities::{transactions::Model as TransactionModel,blocks::Model as BlockModel, logs::Model as LogModel};
use futures::AsyncReadExt;
use sea_orm::DbConn;

use crate::evms::eth::EthCli;
use crate::repo::height::{Mutation, Query};
use std::time::Duration;
use tokio::time::interval;

use ethereum_types::U256;
use web3::types::{Block, BlockNumber};

use config::{db::DB, chain::Chain};
use crate::models::{Event, Transaction};
use crate::rpc::EthRpcClient;

pub async fn init_block() {
    let db = DB::new();
    let block_count = db.block.count().await.unwrap();

    if block_count == 0 {
        let latest_block_number = EthRpcClient::block_number().await.unwrap();
        let latest_block = EthRpcClient::block_by_number(BlockNumber::Number(latest_block_number), false).await.unwrap();

        let block = BlockModel {
            block_hash: latest_block.hash.unwrap().to_string(),
            block_height: latest_block.number.unwrap().as_u64(),
            latest_block_height: latest_block.number.unwrap().as_u64(),
            parent_hash: latest_block.parent_hash.unwrap().to_string(),
        };

        db.block.insert(&block).await.unwrap();
    }
}

// TODO 
pub async fn sync_task(config: Chain) {
    let db = DB::new();
    let mut interval = interval(Duration::from_secs(1));
    let cli = EthCli::new(&config.url.as_str());
    loop {
        interval.tick().await;

        let latest_block_number = cli.get_block_number().await;
        let latest_block = db.block.get_latest().await.unwrap();

        if latest_block.latest_block_height > latest_block_number {
            log::info!(
                "latestBlock.LatestBlockHeight: {} greater than latestBlockNumber: {}",
                latest_block.latest_block_height,
                latest_block_number
            );
            continue;
        }

        let current_block = EthRpcClient::block_by_number(BlockNumber::Number(latest_block.latest_block_height), true)
            .await
            .unwrap();

        log::info!(
            "get currentBlock blockNumber: {}, blockHash: {}",
            current_block.number.unwrap(),
            current_block.hash.unwrap()
        );

        handle_block(&db, &config, &current_block).await.unwrap();
    }
}

async fn handle_block(db: &DB, config: &Config, block: &Block<U256>) -> Result<(), Box<dyn std::error::Error>> {
    let block_model = BlockModel {
        block_height: block.number.unwrap().as_u64(),
        block_hash: block.hash.unwrap().to_string(),
        parent_hash: block.parent_hash.unwrap().to_string(),
        latest_block_height: block.number.unwrap().as_u64() + 1,
    };

    let (events, transactions) = handle_transactions(block).await?;

    sync_to_db(db, &block_model, &transactions, &events).await?;

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

async fn handle_transactions(block: &Block<U256>) -> Result<(Vec<LogModel>, Vec<TransactionModel>), Box<dyn std::error::Error>> {
    let mut events = Vec::new();
    let mut transactions = Vec::new();

    for tx in block.transactions.iter() {
        let receipt = EthRpcClient::transaction_receipt(&tx.hash.unwrap()).await?;

        for log in receipt.logs.iter() {
            let event = handle_transaction_event(log, receipt.status)?;
            events.push(event);
        }

        let transaction = process_transaction(tx, &block.number.unwrap(), receipt.status)?;
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
        log::info!("Contract creation found, Sender: {}, TxHash: {}", transaction.from, transaction.tx_hash);
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

fn handle_transaction_event(log: &web3::types::Log, status: U256) -> Result<LogModel, Box<dyn std::error::Error>> {
    log::info!(
        "ProcessTransactionEvent, address: {}, data: {}",
        log.address,
        hex::encode(&log.data)
    );

    let mut event = EventModel {
        address: log.address.to_string(),
        data: hex::encode(&log.data),
        block_number: log.block_number.unwrap().as_u64(),
        tx_hash: log.transaction_hash.unwrap().to_string(),
        tx_index: log.transaction_index.unwrap().as_u64(),
        block_hash: log.block_hash.unwrap().to_string(),
        log_index: log.log_index.unwrap().as_u64(),
        removed: log.removed.unwrap_or(false),
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
    let code = EthRpcClient::code_at(&addr, None).await?;

    Ok(!code.is_empty())
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
