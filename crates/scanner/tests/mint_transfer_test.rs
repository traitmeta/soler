use entities::address_current_token_balances;
use ethers::types::TransactionReceipt;
use scanner::handler::{event::handle_block_event, token::handle_token_from_receipts};

use std::{env, fs::File, io::BufReader, path::PathBuf};

const TEST_DATA_DIR: &str = "tests/data/";

fn common_dir_path() -> PathBuf {
    let mut current_dir = env::current_dir().unwrap();
    println!(
        "Entries modified in the last 24 hours in {:?}:",
        current_dir
    );
    if !current_dir.ends_with("crates/scanner") {
        current_dir.push("crates/scanner");
    }
    current_dir.push(TEST_DATA_DIR);
    current_dir
}

#[test]
fn test_handle_batch_transfer_1155() {
    let mut current_dir = common_dir_path();
    current_dir.push("receipts/erc20_mint.json");
    let file = File::open(&current_dir).expect(&format!("{}", &current_dir.as_path().display()));
    let reader = BufReader::new(file);
    let receipt: TransactionReceipt = serde_json::from_reader(reader).unwrap();
    let receipts = vec![receipt];

    let events = handle_block_event(&receipts);
    assert!(events.len() >= 1);
}

#[test]
fn test_handle_token_from_receipts() {
    let mut current_dir = common_dir_path();
    current_dir.push("receipts/erc20_transfer.json");
    let file = File::open(&current_dir).expect(&format!("{}", &current_dir.as_path().display()));
    let reader = BufReader::new(file);
    let receipt: TransactionReceipt = serde_json::from_reader(reader).unwrap();
    let receipts = vec![receipt];

    let (tokens, tokens_transfer, address_token_balances, address_current_token_balances) =
        handle_token_from_receipts(&receipts);
    assert!(tokens.len() >= 1);
    assert!(tokens_transfer.len() >= 1);
    assert!(address_token_balances.len() >= 1);
    assert!(address_current_token_balances.len() >= 1);
}

#[test]
fn test_handle_token_from_receipts_deposit() {
    let mut current_dir = common_dir_path();
    current_dir.push("receipts/erc20_deposit.json");
    let file = File::open(&current_dir).expect(&format!("{}", &current_dir.as_path().display()));
    let reader = BufReader::new(file);
    let receipt: TransactionReceipt = serde_json::from_reader(reader).unwrap();
    let receipts = vec![receipt];

    let (tokens, tokens_transfer, address_token_balances, address_current_token_balances) =
        handle_token_from_receipts(&receipts);
    assert!(tokens.len() >= 1);
    assert!(tokens_transfer.len() >= 1);
    assert!(address_token_balances.len() >= 1);
    assert!(address_current_token_balances.len() >= 1);
}
