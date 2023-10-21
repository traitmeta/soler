use ethers::types::{TransactionReceipt, H256};
use scanner::{common::consts, evms::eth::EthCli, handler::token::token_process};
use serde_json::json;
use std::{env, fs::File, io::BufReader};
use tokio::runtime::Runtime;

const TEST_DATA_DIR: &str = "tests/data/";

#[test]
#[ignore = "just for data source"]
fn test_get_data_from_remote() {
    let eth_cli = EthCli::new("https://rpc.ankr.com/eth_goerli");
    let tx_hash: H256 = "0xa13cfb21e8edfe1090da3612f7a6b6834e0e03d642069fe4d3a14645b8dc744a"
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

#[test]
fn test_handle_transfer_20() {
    let mut current_dir = env::current_dir().unwrap();
    println!(
        "Entries modified in the last 24 hours in {:?}:",
        current_dir
    );
    if !current_dir.ends_with("crates/scanner") {
        current_dir.push("crates/scanner");
    }
    current_dir.push(TEST_DATA_DIR);
    current_dir.push("token_handler/erc20.json");
    let file = File::open(&current_dir).expect(&format!("{}", &current_dir.as_path().display()));
    let reader = BufReader::new(file);
    let receipt: TransactionReceipt = serde_json::from_reader(reader).unwrap();

    let (tokens, token_transfers) = token_process(receipt.logs);
    assert!(tokens.len() >= 1);
    assert!(token_transfers.len() >= 1);
    assert!(token_transfers.len() < 3);
    assert!(token_transfers[0].amount.is_some());
    assert!(token_transfers[1].amount.is_some());
    if let Some(amount) = &token_transfers[0].amount {
        assert!(amount.to_string() == "276382516628653");
    };
    if let Some(amount) = &token_transfers[1].amount {
        assert!(amount.to_string() == "6896650579998663500");
    };
}

#[test]
fn test_handle_transfer_weth_deposit() {
    let mut current_dir = env::current_dir().unwrap();
    println!(
        "Entries modified in the last 24 hours in {:?}:",
        current_dir
    );
    if !current_dir.ends_with("crates/scanner") {
        current_dir.push("crates/scanner");
    }
    current_dir.push(TEST_DATA_DIR);
    current_dir.push("token_handler/weth.json");
    let file = File::open(&current_dir).expect(&format!("{}", &current_dir.as_path().display()));
    let reader = BufReader::new(file);
    let receipt: TransactionReceipt = serde_json::from_reader(reader).unwrap();

    let (tokens, token_transfers) = token_process(receipt.logs);
    assert!(tokens.len() == 2);
    assert!(token_transfers.len() == 4);
    assert!(token_transfers[0].from_address_hash == consts::ZERO_ADDRESS.as_bytes().to_vec());
    assert!(token_transfers[0].amount.is_some());
    assert!(token_transfers[2].amount.is_some());
    if let Some(amount) = &token_transfers[0].amount {
        assert!(amount.to_string() == "1000000000000000000");
    };
    if let Some(amount) = &token_transfers[2].amount {
        assert!(amount.to_string() == "3644233048165313285693");
    };
}
