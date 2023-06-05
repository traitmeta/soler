use web3::{
    transports::Http,
    types::{BlockId, BlockNumber, Log, H256, U64},
    Web3,
};

pub async fn batch_get_tx_logs(
    current_height: u64,
    web3: &Web3<Http>,
) -> (Option<web3::types::U256>, Option<Vec<web3::types::Log>>) {
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

    let mut logs: Vec<Log> = Vec::new();
    for v in block_info.transactions {
        if let Some(receipt) = get_tx_logs_by_id(web3, v).await{
            logs.extend(receipt.logs.into_iter());
        }
    }

    return (Some(block_info.timestamp), Some(logs));
}

pub async fn get_tx_logs(
    current_height: u64,
    web3: &Web3<Http>,
) -> (
    Option<web3::types::U256>,
    Option<web3::types::TransactionReceipt>,
) {
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

    let receipt = get_tx_logs_by_id(web3, block_info.transactions[0])
        .await
        .expect("get teceipt err");

    for i in receipt.logs.iter() {
        tracing::debug!(
            "tx_hash:{:?}, receipt_log: {:?}",
            block_info.transactions[0],
            *i
        );
    }

    return (Some(block_info.timestamp), Some(receipt));
}

pub async fn get_tx_logs_by_id(
    web3: &Web3<Http>,
    hash: H256,
) -> Option<web3::types::TransactionReceipt> {
    web3.eth().transaction_receipt(hash).await.unwrap()
}

pub async fn get_block_info_by_height(
    web3: &Web3<Http>,
    height: U64,
) -> Option<web3::types::Block<H256>> {
    web3.eth()
        .block(BlockId::Number(BlockNumber::Number(height)))
        .await
        .expect("get block info err")
}
