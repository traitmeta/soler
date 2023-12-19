use ethers::types::TransactionReceipt;
use scanner::handler::event::handle_block_event;

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
