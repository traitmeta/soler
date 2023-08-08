use web3::{
    transports::Http,
    types::{
        Block, BlockId, BlockNumber, Bytes, Index, Log, TransactionReceipt, H160, H256, U256, U64,
    },
    Web3,
};

use crate::handler::block;

/*
    1. event需要分开，下游的接收者只关心某一类event
        通过address判断是什么交易
    2. event需要按照TRX的维度聚合成一个，下游的接收者关心的事件有上下文依赖
        a. 如何辨别这个是什么交易？比如是注册用户域名？还是撮合NFT交易？
        b. 可以在过滤的时候增加一个合约地址，在过滤交易事件的时候，把to地址匹配一下，如果正确就是需要过滤的交易信息
*/

/// A log produced by a transaction.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MyLog {
    #[serde(rename = "logDetails")]
    pub log_details: Vec<LogDetail>,
    /// Block Hash
    #[serde(rename = "blockHash")]
    pub block_hash: H256,
    /// Block Number
    #[serde(rename = "blockNumber")]
    pub block_number: U64,
    #[serde(rename = "blockTimestamp")]
    pub block_timestamp: U256,
    /// Transaction Index
    #[serde(rename = "transactionIndex")]
    pub transaction_index: Index,
    // transaction to address
    pub to: H160,
}

/// A log produced by a transaction.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LogDetail {
    /// H160
    pub address: H160,
    /// Topics
    pub topics: Vec<H256>,
    /// Data
    pub data: Bytes,
    /// Log Index in Transaction
    #[serde(rename = "transactionLogIndex")]
    pub transaction_log_index: Option<U256>,
    /// Log Index in Block
    #[serde(rename = "logIndex")]
    pub log_index: Option<U256>,
    /// Log Type
    #[serde(rename = "logType")]
    pub log_type: Option<String>,
    /// Removed
    pub removed: Option<bool>,
}

pub async fn batch_get_tx_logs(current_height: u64, web3: &Web3<Http>) -> (Option<U256>, Vec<Log>) {
    let height = web3
        .eth()
        .block_number()
        .await
        .expect("get block number err");
    println!("{}", height);
    if current_height >= height.as_u64() {
        return (None, None);
    }
    let block_info = match get_block_info_by_height(web3, height).await {
        Some(it) => it,
        _ => return (None, None),
    };
    tracing::debug!("timestamp: {}", block_info.timestamp);
    tracing::debug!("txs: {:?}", block_info.transactions);

    let mut logs: Vec<MyLog> = Vec::new();
    for trx in block_info.transactions {
        if let Some(receipt) = get_tx_logs_by_id(web3, trx).await {
            for l in receipt.logs.iter() {
                let mut log_details: Vec<LogDetail> = Vec::new();
                let mut my_log: MyLog = MyLog {
                    log_details,
                    block_hash: block_info.hash,
                    block_number: block_info.number,
                    block_timestamp: block_info.timestamp,
                    transaction_index: receipt.transaction_index,
                    to: receipt.to.unwrap(),
                };
                let detail = LogDetail {
                    address: l.address,
                    topics: l.topics,
                    data: l.data,
                    transaction_log_index: l.transaction_log_index,
                    log_index: l.log_index,
                    log_type: l.log_type,
                    removed: l.removed,
                };
                log_details.append(detail);
                logs.append(my_log);
            }
        }
        return;
    }

    return (Some(block_info.timestamp), Some(logs));
}

pub async fn get_tx_logs(current_height: u64, web3: &Web3<Http>) -> Vec<MyLog> {
    let height = web3
        .eth()
        .block_number()
        .await
        .expect("get block number err");
    tracing::debug!("current height {}", height);
    if current_height >= height.as_u64() {
        return (None, None);
    }

    let block_info = match get_block_info_by_height(web3, height).await {
        Some(it) => it,
        _ => return (None, None),
    };
    tracing::debug!("timestamp: {}", block_info.timestamp);
    tracing::debug!("txs: {:?}", block_info.transactions);

    let mut logs: Vec<MyLog> = Vec::new();
    for trx in block_info.transactions {
        if let Some(receipt) = get_tx_logs_by_id(web3, trx).await {
            let mut log_details: Vec<LogDetail> = Vec::new();
            let mut my_log: MyLog = MyLog {
                log_details,
                block_hash: block_info.hash,
                block_number: block_info.number,
                block_timestamp: block_info.timestamp,
                transaction_index: receipt.transaction_index,
                to: receipt.to.unwrap(),
            };

            for l in receipt.logs.iter() {
                let detail = LogDetail {
                    address: l.address,
                    topics: l.topics,
                    data: l.data,
                    transaction_log_index: l.transaction_log_index,
                    log_index: l.log_index,
                    log_type: l.log_type,
                    removed: l.removed,
                };
                log_details.append(detail);
            }

            logs.append(my_log);
        }
    }

    return logs;
}

pub async fn get_tx_logs_by_id(web3: &Web3<Http>, hash: H256) -> Option<TransactionReceipt> {
    web3.eth().transaction_receipt(hash).await.unwrap()
}

pub async fn get_block_info_by_height(web3: &Web3<Http>, height: U64) -> Option<Block<H256>> {
    web3.eth()
        .block(BlockId::Number(BlockNumber::Number(height)))
        .await
        .expect("get block info err")
}
