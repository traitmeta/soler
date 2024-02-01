use bigdecimal::FromPrimitive;
use ethers::types::{Block, Trace, Transaction, TransactionReceipt};
use scanner::handler::{
    address::process_block_addresses,
    block::handle_block_header,
    internal_transaction::{classify_txs, handler_inner_transaction},
    transaction::handle_transactions,
};
use sea_orm::prelude::Decimal;
use std::{collections::HashMap, env, fs::File, io::BufReader, path::PathBuf};

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
fn test_handle_block_header() {
    let mut current_dir = common_dir_path();
    current_dir.push("blocks/with_txdetails.json");
    let file = File::open(&current_dir).expect(&format!("{}", &current_dir.as_path().display()));
    let reader = BufReader::new(file);
    let block: Block<Transaction> = serde_json::from_reader(reader).unwrap();

    let block_header = handle_block_header(&block);
    match block_header {
        Ok(header) => {
            println!("Block header: {:?}", header);
            assert!(header.is_empty.is_some());
            assert!(header.gas_limit == Decimal::from_i32(8000000).unwrap());
            assert!(header.is_empty == Some(false));
            assert!(header.size == Some(4886));
        }
        Err(_) => {
            assert!(false);
            println!("Block header: None");
        }
    }
}

#[test]
fn test_handle_transactions() {
    let mut current_dir = common_dir_path();
    current_dir.push("blocks/with_txdetails.json");
    let file = File::open(&current_dir).expect(&format!("{}", &current_dir.as_path().display()));
    let reader = BufReader::new(file);
    let block: Block<Transaction> = serde_json::from_reader(reader).unwrap();

    let mut current_dir = common_dir_path();
    current_dir.push("blocks/recipts.json");
    let file = File::open(&current_dir).expect(&format!("{}", &current_dir.as_path().display()));
    let reader = BufReader::new(file);
    let recipts: Vec<TransactionReceipt> = serde_json::from_reader(reader).unwrap();

    let mut current_dir = common_dir_path();
    current_dir.push("blocks/traces.json");
    let file = File::open(&current_dir).expect(&format!("{}", &current_dir.as_path().display()));
    let reader = BufReader::new(file);
    let traces: Vec<Trace> = serde_json::from_reader(reader).unwrap();

    let recipet_map = recipts
        .iter()
        .map(|r| (r.transaction_hash, r.clone()))
        .collect::<HashMap<_, _>>();

    let trace_map = classify_txs(traces.as_slice());

    let trxes = handle_transactions(&block, &recipet_map, &trace_map);
    match trxes {
        Ok(txs) => {
            println!("Transactions: {:?}", txs);
            assert!(txs.len() == 13);
        }
        Err(e) => {
            assert!(false);
            println!("Transactions: err: {}", e);
        }
    }
}

#[test]
fn test_handle_addresses() {
    let mut current_dir = common_dir_path();
    current_dir.push("blocks/with_txdetails.json");
    let file = File::open(&current_dir).expect(&format!("{}", &current_dir.as_path().display()));
    let reader = BufReader::new(file);
    let block: Block<Transaction> = serde_json::from_reader(reader).unwrap();

    let mut current_dir = common_dir_path();
    current_dir.push("blocks/recipts.json");
    let file = File::open(&current_dir).expect(&format!("{}", &current_dir.as_path().display()));
    let reader = BufReader::new(file);
    let recipts: Vec<TransactionReceipt> = serde_json::from_reader(reader).unwrap();

    let mut current_dir = common_dir_path();
    current_dir.push("blocks/traces.json");
    let file = File::open(&current_dir).expect(&format!("{}", &current_dir.as_path().display()));
    let reader = BufReader::new(file);
    let traces: Vec<Trace> = serde_json::from_reader(reader).unwrap();

    let recipet_map = recipts
        .iter()
        .map(|r| (r.transaction_hash, r.clone()))
        .collect::<HashMap<_, _>>();

    let trace_map = classify_txs(traces.as_slice());

    let addresses = process_block_addresses(&block, &recipet_map, &trace_map);
    assert!(addresses.len() == 21);
}

#[test]
fn test_handle_inner_tx() {
    let mut current_dir = common_dir_path();
    current_dir.push("blocks/traces.json");
    let file = File::open(&current_dir).expect(&format!("{}", &current_dir.as_path().display()));
    let reader = BufReader::new(file);
    let traces: Vec<Trace> = serde_json::from_reader(reader).unwrap();

    let inner_tx = handler_inner_transaction(&traces);
    assert!(inner_tx.len() == 40);
}
