use serde::{Deserialize, Serialize};
use web3::{
    transports::Http,
    types::{
        Address, Block, BlockId, BlockNumber, Bytes, Filter, FilterBuilder, Index, Log,
        Transaction, TransactionReceipt, H160, H256, U256, U64,
    },
    Web3,
};

use crate::{cache::ContractAddrCache, handler::block};

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

pub struct EthCli {
    web3: Web3<Http>,
}

impl EthCli {
    pub fn new(url: &str) -> EthCli {
        let transport = Http::new(url).unwrap();
        EthCli {
            web3: Web3::new(transport),
        }
    }

    pub async fn get_block_number(&self) -> u64 {
        let block_number = self.web3.eth().block_number().await.unwrap();
        block_number.as_u64()
    }
    pub async fn get_block(&self, block_number: u64) -> Block<H256> {
        let block_number = BlockNumber::Number(block_number.into());
        let block = self
            .web3
            .eth()
            .block(web3::types::BlockId::Number(block_number))
            .await
            .unwrap();
        block.unwrap()
    }

    pub async fn get_block_with_tx(&self, block_number: u64) -> Block<Transaction> {
        let block_number = BlockNumber::Number(block_number.into());
        let block = self
            .web3
            .eth()
            .block_with_txs(web3::types::BlockId::Number(block_number))
            .await
            .unwrap();
        block.unwrap()
    }

    pub async fn get_block_by_hash(&self, block_hash: H256) -> Block<H256> {
        let block = self
            .web3
            .eth()
            .block(web3::types::BlockId::Hash(block_hash))
            .await
            .unwrap();
        block.unwrap()
    }
    pub async fn get_transaction_receipt(
        &self,
        transaction_hash: H256,
    ) -> Option<TransactionReceipt> {
        let receipt = self
            .web3
            .eth()
            .transaction_receipt(transaction_hash)
            .await
            .unwrap();
        receipt
    }
    pub async fn get_transaction(&self, transaction_hash: H256) -> Transaction {
        let transaction = self
            .web3
            .eth()
            .transaction(web3::types::TransactionId::Hash(transaction_hash))
            .await
            .unwrap();
        transaction.unwrap()
    }
    pub async fn get_logs(
        &self,
        topic1: Option<Vec<H256>>,
        topic2: Option<Vec<H256>>,
        topic3: Option<Vec<H256>>,
        topic4: Option<Vec<H256>>,
    ) -> Vec<Log> {
        let filter = FilterBuilder::default()
            .topics(topic1, topic2, topic3, topic4)
            .build();
        let logs = self.web3.eth().logs(filter).await.unwrap();
        logs
    }

    pub async fn code_at(&self, address: Address, block_number: U64) -> Bytes {
        let code = self
            .web3
            .eth()
            .code(
                address,
                Some(web3::types::BlockNumber::Number(block_number)),
            )
            .await
            .unwrap();
        code
    }

    pub async fn batch_get_tx_logs(
        &self,
        block_info: Block<Transaction>,
    ) -> (Option<U256>, Vec<MyLog>) {
        let mut logs: Vec<MyLog> = Vec::new();
        for trx in block_info.transactions {
            if let Some(receipt) = self.get_transaction_receipt(trx.hash).await {
                let my_log = MyLog {
                    block_hash: block_info.hash.unwrap(),
                    block_number: block_info.number.unwrap(),
                    block_timestamp: block_info.timestamp,
                    transaction_index: receipt.transaction_index,
                    to: receipt.to.unwrap(),
                    log_details: Vec::new(),
                };

                for l in receipt.logs.iter() {
                    let mut log_details: Vec<LogDetail> = Vec::new();
                    let detail = LogDetail {
                        address: l.address,
                        topics: l.topics.clone(),
                        data: l.data.clone(),
                        transaction_log_index: l.transaction_log_index,
                        log_index: l.log_index,
                        log_type: l.log_type.clone(),
                        removed: l.removed,
                    };
                    log_details.push(detail);
                }
                logs.push(my_log);
            }
        }

        return (Some(block_info.timestamp), logs);
    }
}
