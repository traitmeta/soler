use chrono::Utc;
use entities::addresses::Model as AddressModel;
use ethers::types::{ActionType, Res, Trace, Transaction, TransactionReceipt};

pub fn process_addresses(
    tx: &Transaction,
    receipt: Option<TransactionReceipt>,
    traces: Option<Vec<(Trace, i32)>>,
) -> anyhow::Result<Vec<AddressModel>> {
    tracing::info!("hand addresses, txHash: {:#032x}", tx.hash);
    let mut addresses = vec![];
    let from_address = AddressModel {
        fetched_coin_balance: None,
        fetched_coin_balance_block_number: None,
        hash: tx.from.as_bytes().to_vec(),
        contract_code: None,
        inserted_at: Utc::now().naive_utc(),
        updated_at: Utc::now().naive_utc(),
        nonce: Some(tx.nonce.as_u64() as i32),
        decompiled: None,
        verified: None,
        gas_used: None,
        transactions_count: Some(tx.nonce.as_u64() as i32),
        token_transfers_count: None,
    };

    addresses.push(from_address);

    // to exist: if tx input start with safeCreate2 method id, contract code get from trace
    // to not exist: address from receipt and contract code from trace
    let code = get_contract_code_from_trace(traces);
    match tx.to {
        Some(to) => {
            let to_address = AddressModel {
                fetched_coin_balance: None,
                fetched_coin_balance_block_number: None,
                hash: to.as_bytes().to_vec(),
                contract_code: code.map(|code| code.into_bytes()),
                inserted_at: Utc::now().naive_utc(),
                updated_at: Utc::now().naive_utc(),
                nonce: None,
                decompiled: None,
                verified: None,
                gas_used: None,
                transactions_count: None,
                token_transfers_count: None,
            };
            addresses.push(to_address);
        }
        None => {
            let to = get_contract_address_from_receipt(receipt).unwrap();
            let create_address = AddressModel {
                fetched_coin_balance: None,
                fetched_coin_balance_block_number: None,
                hash: to,
                contract_code: code.map(|code| code.into_bytes()),
                inserted_at: Utc::now().naive_utc(),
                updated_at: Utc::now().naive_utc(),
                nonce: None,
                decompiled: None,
                verified: None,
                gas_used: None,
                transactions_count: None,
                token_transfers_count: None,
            };
            addresses.push(create_address);
        }
    }

    Ok(addresses)
}

fn get_contract_address_from_receipt(receipt: Option<TransactionReceipt>) -> Option<Vec<u8>> {
    match receipt {
        Some(receipt) => {
            if let Some(contract_address) = receipt.contract_address {
                return Some(contract_address.as_bytes().to_vec());
            };
            if let Some(to) = receipt.to {
                return Some(to.as_bytes().to_vec());
            };
        }
        None => return None,
    };

    None
}

fn get_contract_code_from_trace(traces: Option<Vec<(Trace, i32)>>) -> Option<String> {
    if let Some(traces) = traces {
        for (trace, _) in traces.iter() {
            if trace.action_type == ActionType::Create {
                match &trace.result {
                    Some(result) => match result {
                        Res::Create(res) => Some(res.code.to_vec()),
                        _ => None,
                    },
                    None => None,
                };
            }
        }
    }

    None
}
