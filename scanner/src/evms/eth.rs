use ethers::providers::{Middleware, Provider};
use ethers::types::{
    Address, Block, BlockId, BlockNumber, Bytes, Trace, Transaction, TransactionReceipt, TxHash,
    H160, H256, U256, U64,
};
use serde::{Deserialize, Serialize};

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
    pub transaction_index: U64,
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
    provider: Provider<ethers::providers::Http>,
}

// TODO trace fail transaction
impl EthCli {
    pub fn new(url: &str) -> EthCli {
        let provider = Provider::try_from(url).unwrap();
        EthCli { provider }
    }

    pub async fn get_block_number(&self) -> u64 {
        let block_number = self.provider.get_block_number().await.unwrap();
        block_number.as_u64()
    }

    pub async fn get_block(&self, block_number: u64) -> Block<TxHash> {
        let block = self
            .provider
            .get_block(BlockNumber::Number(block_number.into()))
            .await
            .unwrap();
        block.unwrap()
    }

    pub async fn get_block_with_tx(&self, block_number: u64) -> Block<Transaction> {
        let block = self
            .provider
            .get_block_with_txs(BlockNumber::Number(block_number.into()))
            .await
            .unwrap();
        block.unwrap()
    }

    pub async fn get_block_by_hash(&self, block_hash: H256) -> Block<TxHash> {
        let block = self
            .provider
            .get_block(BlockId::Hash(block_hash))
            .await
            .unwrap();
        block.unwrap()
    }

    pub async fn get_block_receipt(&self, block_number: u64) -> Vec<TransactionReceipt> {
        let receipts = self
            .provider
            .get_block_receipts(BlockNumber::Number(block_number.into()))
            .await
            .unwrap();
        receipts
    }

    pub async fn get_transaction_receipt(
        &self,
        transaction_hash: H256,
    ) -> Option<TransactionReceipt> {
        let receipt = self
            .provider
            .get_transaction_receipt(transaction_hash)
            .await
            .unwrap();
        receipt
    }

    pub async fn get_transaction(&self, transaction_hash: H256) -> Transaction {
        let transaction = self
            .provider
            .get_transaction(transaction_hash)
            .await
            .unwrap();
        transaction.unwrap()
    }

    pub async fn code_at(&self, address: Address, block_number: U64) -> Bytes {
        let code = self
            .provider
            .get_code(address, Some(BlockId::Number(block_number.into())))
            .await
            .unwrap();
        code
    }

    pub async fn trace_transaction(&self, transaction_hash: H256) -> Vec<Trace> {
        let trace = self
            .provider
            .trace_transaction(transaction_hash)
            .await
            .unwrap();
        trace
    }

    pub async fn trace_block(&self, number: u64) -> Vec<Trace> {
        let trace = self
            .provider
            .trace_block(BlockNumber::Number(number.into()))
            .await
            .unwrap();
        trace
    }
   
    pub async fn batch_get_tx_logs(
        &self,
        block_info: Block<Transaction>,
    ) -> (Option<U256>, Vec<MyLog>) {
        let mut logs: Vec<MyLog> = Vec::new();
        if let Some(block_number) = block_info.number {
            let receipts = self.get_block_receipt(block_number.as_u64()).await;
            for receipt in receipts {
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
