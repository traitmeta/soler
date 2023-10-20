use ethers::{
    abi::token,
    types::{TransactionReceipt, H256},
};
use scanner::{evms::eth::EthCli, handler::token::token_process};
use serde_json::json;
use std::{env, fs::File, io::BufReader, path::PathBuf};
use tokio::runtime::Runtime;

const TEST_DATA_DIR: &str = "tests/data/";

#[test]
#[ignore = "just for data source"]
fn test_get_data_from_remote() {
    let eth_cli = EthCli::new("https://rpc.ankr.com/eth_goerli");
    let tx_hash: H256 = "0x0e3d6999d6f8ce500a0acbbc4efd723625d9bb7df4a4f9be33c1317316b213d0"
        .parse()
        .unwrap();

    let runtime = Runtime::new().unwrap();
    let recipet = runtime
        .block_on(eth_cli.get_transaction_receipt(tx_hash))
        .unwrap();
    json!(recipet);
    let json_str = serde_json::to_string(&recipet).unwrap();
    println!("{}", json_str);
}

#[test]
fn test_handle_batch_transfer_1155() {
    let mut current_dir = env::current_dir().unwrap();
    println!(
        "Entries modified in the last 24 hours in {:?}:",
        current_dir
    );
    if !current_dir.ends_with("crates/scanner") {
        current_dir.push("crates/scanner");
    }
    current_dir.push(TEST_DATA_DIR);
    current_dir.push("token_handler/erc1155_batch.json");
    let file = File::open(&current_dir).expect(&format!("{}", &current_dir.as_path().display()));
    let reader = BufReader::new(file);
    let receipt: TransactionReceipt = serde_json::from_reader(reader).unwrap();

    let (tokens, token_transfers) = token_process(receipt.logs);
    assert!(tokens.len() >= 1);
    assert!(token_transfers.len() >= 1);
    assert!(token_transfers[0].token_ids.is_some());
    if let Some(token_ids) = &token_transfers[0].token_ids {
        println!("{}", token_ids.len());
        assert!(token_ids.len() == 35);
    };
}
