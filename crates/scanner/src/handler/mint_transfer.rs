use ethers::types::{Log, TransactionReceipt, H160};

use common::{chain_ident, consts};

pub struct MintTransfer {
    pub from: Vec<u8>,
    pub to: Vec<u8>,
    pub block_number: u64,
}

pub fn parse_mint_transfer(receipts: &[TransactionReceipt]) -> Vec<MintTransfer> {
    let mut mint_transfers = vec![];
    for receipt in receipts.iter() {
        let mut tmp = parse(&receipt.logs);
        mint_transfers.append(&mut tmp);
    }

    mint_transfers
}

fn parse(logs: &[Log]) -> Vec<MintTransfer> {
    let mut mint_transfers: Vec<MintTransfer> = Vec::new();

    for log in logs.iter() {
        let first_topic = chain_ident!(log.topics[0].as_bytes());
        if first_topic.as_str() == consts::BRIDGE_HASH {
            if let Some(params) = parse_params(log) {
                mint_transfers.push(params);
            }
        }
    }

    mint_transfers
}

fn parse_params(log: &Log) -> Option<MintTransfer> {
    if let (second_topic, third_topic, Some(block_number)) =
        (log.topics[1], log.topics[2], log.block_number)
    {
        let to = H160::from(second_topic).as_bytes().to_vec();
        let from = H160::from(third_topic).as_bytes().to_vec();
        return Some(MintTransfer {
            block_number: block_number.as_u64(),
            from,
            to,
        });
    }

    None
}

#[cfg(test)]
mod tests {
    use ethers::types::{Log, H160};

    use super::parse;

    #[test]
    fn test_parse() {
        let src = r#"
        [
            {
                "address": "0x867305d19606aadba405ce534e303d0e225f9556",
                "topics": [
                    "0x3c798bbcf33115b42c728b8504cff11dd58736e9fa789f1cda2738db7d696b2a",
                    "0x0000000000000000000000009a4a90e2732f3fa4087b0bb4bf85c76d14833df1",
                    "0x0000000000000000000000007301cfa0e1756b71869e93d4e4dca5c7d0eb0aa6"
                ],
                "data": "0x0000000000000000000000000000000000000000000000001bc16d674ec80000",
                "blockHash": "0x1000e552ce843557bbf120df4d075fae2b56b0f57f3621b88dffc9a210ef223e",
                "blockNumber": "0x972413",
                "transactionHash": "0x1d5066d30ff3404a9306733136103ac2b0b989951c38df637f464f3667f8d4ee",
                "transactionIndex": "0xc",
                "logIndex": "0x34",
                "removed": false
            }
        ]
        "#;
        let log: Vec<Log> = serde_json::from_str(src).unwrap();
        let parsed_data = parse(&log);
        assert!(parsed_data.len() == 1);
        let to: H160 = "0x9a4a90e2732f3fa4087b0bb4bf85c76d14833df1"
            .parse()
            .unwrap();
        let from: H160 = "0x7301cfa0e1756b71869e93d4e4dca5c7d0eb0aa6"
            .parse()
            .unwrap();
        assert!(parsed_data[0].from == from.as_bytes().to_vec());
        assert!(parsed_data[0].to == to.as_bytes().to_vec());
    }
}
