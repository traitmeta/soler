use bigdecimal::FromPrimitive;
use ethers::types::{Block, Transaction};
use scanner::handler::block::handle_block_header;
use sea_orm::prelude::Decimal;

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
fn test_handle_block_header() {
    let mut current_dir = common_dir_path();
    current_dir.push("blocks/block_with_txdetails.json");
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
