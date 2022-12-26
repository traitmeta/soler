// 使用EVM的签名
use ethers_core::{k256::ecdsa::SigningKey, rand::thread_rng, utils::keccak256};
use ethers_signers::{to_eip155_v, LocalWallet, Signer, Wallet};
use std::io::Read;
use web3::{
    transports::Http,
    types::{BlockId, BlockNumber, Log, H256, U64},
    Web3,
};

pub fn random_wallet() -> Wallet<SigningKey> {
    let wallet = LocalWallet::new(&mut thread_rng());
    let wallet = wallet.with_chain_id(1u64);
    wallet
}

pub fn sign_msg(wallet: Wallet<SigningKey>) {
    let digest = md5::compute(b"\"hello2\"");
    let k256 = keccak256(&digest[0..8]).into();
    let sig = wallet.sign_hash(k256); //里面有对recover_id加27操作
    to_eip155_v(sig.v as u8 - 27, 1); // sig.v = sig.v - 27;
    let signstr = sig.to_vec();
    println!("{:?} {:?}", digest.bytes(), hex::encode(signstr));
}

pub async fn sign_hash(wallet: Wallet<SigningKey>) {
    // const PREFIX: &str = "\x19Ethereum Signed Message:\n";
    let signature = wallet.sign_message("hello world").await.unwrap();
    signature.verify("hello world", wallet.address()).unwrap()
}

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
