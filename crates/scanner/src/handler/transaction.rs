use std::{collections::HashMap, str::FromStr};

use anyhow::bail;
use chrono::Utc;
use entities::transactions::Model as TransactionModel;
use ethers::types::{Block, Trace, Transaction, TransactionReceipt, H256, U64};
use sea_orm::prelude::{BigDecimal, Decimal};

use crate::common::err::ScannerError;

pub async fn handle_transactions(
    block: &Block<Transaction>,
    recipt_map: &HashMap<H256, TransactionReceipt>,
    trace_map: &HashMap<H256, Vec<(Trace, i32)>>,
) -> anyhow::Result<Vec<TransactionModel>> {
    let mut transactions = Vec::new();
    for tx in block.transactions.iter() {
        let recipt = recipt_map.get(&tx.hash);
        let traces = trace_map.get(&tx.hash);
        let transaction = process_transaction(tx, &block.number, recipt, traces).await?;
        transactions.push(transaction);
    }

    Ok(transactions)
}

async fn process_transaction(
    tx: &Transaction,
    block_number: &Option<U64>,
    receipt: Option<&TransactionReceipt>,
    traces: Option<&Vec<(Trace, i32)>>,
) -> anyhow::Result<TransactionModel> {
    // tracing::debug!("hand transaction, tx: {:?}", tx);
    tracing::info!("hand transaction, txHash: {:#032x}", tx.hash);

    let mut transaction = TransactionModel {
        block_number: block_number.map(|number| number.as_u64() as i32),
        hash: tx.hash.as_bytes().to_vec(),
        value: match BigDecimal::from_str(tx.value.to_string().as_str()) {
            Ok(dec) => dec,
            Err(err) => bail!(ScannerError::NewDecimal {
                src: "Pocess transaction value".to_string(),
                err: err.to_string()
            }),
        },
        status: receipt
            .as_ref()
            .map(|r| r.status)
            .and_then(|status| status.map(|s| s.as_u64() as i32)),
        cumulative_gas_used: receipt
            .as_ref()
            .map(|r| r.cumulative_gas_used)
            .map(|c| Decimal::from_i128_with_scale(c.as_usize() as i128, 0)),
        error: None,
        gas: Decimal::from_i128_with_scale(tx.gas.as_usize() as i128, 0),
        gas_price: tx
            .gas_price
            .map(|price| Decimal::from_i128_with_scale(price.as_usize() as i128, 0)),
        gas_used: receipt.as_ref().map(|r| r.gas_used).and_then(|gas_used| {
            gas_used.map(|used| Decimal::from_i128_with_scale(used.as_usize() as i128, 0))
        }),
        index: tx.transaction_index.map(|index| index.as_u64() as i32),
        input: tx.input.to_vec(),
        nonce: tx.nonce.as_u64() as i32,
        r: tx.r.to_string().into_bytes(),
        s: tx.s.to_string().into_bytes(),
        v: Decimal::new(tx.v.as_u32() as i64, 0),
        inserted_at: Utc::now().naive_utc(),
        updated_at: Utc::now().naive_utc(),
        block_hash: tx.block_hash.map(|hash| hash.as_bytes().to_vec()),
        from_address_hash: tx.from.as_bytes().to_vec(),
        to_address_hash: tx.to.map(|to| to.as_bytes().to_vec()),
        created_contract_address_hash: None,
        created_contract_code_indexed_at: None,
        earliest_processing_start: None,
        old_block_hash: None,
        revert_reason: None,
        max_priority_fee_per_gas: tx
            .max_priority_fee_per_gas
            .map(|fee| Decimal::from_i128_with_scale(fee.as_usize() as i128, 0)),

        max_fee_per_gas: tx
            .max_fee_per_gas
            .map(|fee| Decimal::from_i128_with_scale(fee.as_usize() as i128, 0)),

        r#type: receipt
            .as_ref()
            .map(|r| r.transaction_type)
            .and_then(|op_t| op_t.map(|t| t.as_u64() as i32)),

        has_error_in_internal_txs: None,
    };

    match &receipt {
        Some(receipt) => {
            if tx.to.is_none() {
                // let to_address = ethers::utils::get_contract_address(tx.from, tx.nonce).to_string();
                if let Some(contract_address) = receipt.contract_address {
                    transaction.created_contract_address_hash =
                        Some(contract_address.as_bytes().to_vec());
                }
                if let Some(to) = receipt.to {
                    transaction.created_contract_address_hash = Some(to.as_bytes().to_vec());
                }
                transaction.created_contract_code_indexed_at = Some(Utc::now().naive_utc())
            }

            if let Some(status) = receipt.status {
                if status.is_zero() {
                    // This is inner transaction
                    if let Some(trace_list) = traces {
                        for (trace, _) in trace_list.iter() {
                            transaction.error = trace.error.clone().map(|e| e.to_string());
                            transaction.revert_reason = trace.result.clone().map(|result| {
                                serde_json::to_string(&result)
                                    .unwrap_or(String::from("Error serializing value"))
                            });
                        }
                    }
                }
            }
        }
        None => {}
    }

    Ok(transaction)
}
