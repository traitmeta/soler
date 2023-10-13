use chrono::Utc;
use entities::internal_transactions::Model;
use ethers::types::{Action, ActionType, CallType, Res, Trace};
use sea_orm::prelude::Decimal;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::{collections::HashMap, str::FromStr};

use crate::model;

/*
* `block_number` - the `t:Explorer.Chain.Block.t/0` `number` that the `transaction` is collated into.
* `call_type` - the type of call.  `nil` when `type` is not `:call`.
* `created_contract_code` - the code of the contract that was created when `type` is `:create`.
* `error` - error message when `:call` or `:create` `type` errors
* `from_address` - the source of the `value`
* `from_address_hash` - hash of the source of the `value`
* `gas` - the amount of gas allowed
* `gas_used` - the amount of gas used.  `nil` when a call errors.
* `index` - the index of this internal transaction inside the `transaction`
* `init` - the constructor arguments for creating `created_contract_code` when `type` is `:create`.
* `input` - input bytes to the call
* `output` - output bytes from the call.  `nil` when a call errors.
* `to_address` - the sink of the `value`
* `to_address_hash` - hash of the sink of the `value`
* `trace_address` - list of traces
* `transaction` - transaction in which this internal transaction occurred
* `transaction_hash` - foreign key for `transaction`
* `transaction_index` - the `t:Explorer.Chain.Transaction.t/0` `index` of `transaction` in `block_number`.
* `type` - type of internal transaction
* `value` - value of transferred from `from_address` to `to_address`
* `block` - block in which this internal transaction occurred
* `block_hash` - foreign key for `block`
* `block_index` - the index of this internal transaction inside the `block`
* `pending_block` - `nil` if `block` has all its internal transactions fetched
*/

pub fn handler_inner_transaction(traces: Vec<Trace>) -> Vec<Model> {
    let classified_trace = classify_txs(traces);
    process_inner_transaction(classified_trace)
}

fn classify_txs(internal_transactions: Vec<Trace>) -> HashMap<String, Vec<(Trace, i32)>> {
    let mut tx_map: HashMap<String, Vec<(Trace, i32)>> = HashMap::new();
    for i in 0..internal_transactions.len() {
        let tx = &internal_transactions[i];
        if let Some(hash) = tx.transaction_hash {
            match tx_map.get_mut(&hash.to_string()) {
                Some(map) => map.push((tx.clone(), i as i32)),
                None => {
                    tx_map.insert(hash.to_string(), vec![(tx.clone(), i as i32)]);
                }
            }
        }
    }

    tx_map
}

fn process_inner_transaction(traces: HashMap<String, Vec<(Trace, i32)>>) -> Vec<Model> {
    let mut res = vec![];

    for (key, val) in traces.iter() {
        for idx in 0..val.len() {
            let model = internal_transaction_to_model(&val[idx], idx as i32);
            res.push(model);
        }
    }

    res
}

fn internal_transaction_to_model(transaction: &(Trace, i32), idx: i32) -> Model {
    let (trace, block_idx) = transaction;
    let mut model = Model {
        call_type: None,
        created_contract_code: None,
        error: trace.error.clone(),
        gas: None,
        gas_used: None,
        index: idx,
        init: None,
        input: None,
        output: None,
        trace_address: trace.trace_address.iter().map(|&f| f as i32).collect(),
        r#type: "".to_string(),
        value: Decimal::ZERO,
        inserted_at: Utc::now().naive_utc(),
        updated_at: Utc::now().naive_utc(),
        created_contract_address_hash: None,
        from_address_hash: None,
        to_address_hash: None,
        transaction_hash: match trace.transaction_hash {
            Some(hash) => hash.as_bytes().to_vec(),
            None => Vec::new(),
        },
        block_number: Some(trace.block_number as i32),
        transaction_index: match trace.transaction_position {
            Some(position) => Some(position as i32),
            None => None,
        },
        block_hash: trace.block_hash.as_bytes().to_vec(),
        block_index: *block_idx,
    };

    match trace.action_type {
        ActionType::Call => {
            model.r#type = "call".to_string();

            match &trace.action {
                Action::Call(call) => {
                    let call_typ = serde_json::to_string(&call.call_type).unwrap();
                    model.call_type = Some(call_typ);
                    model.from_address_hash = Some(call.from.as_bytes().to_vec());
                    model.to_address_hash = Some(call.to.as_bytes().to_vec());
                    model.gas = Some(Decimal::from_i128_with_scale(
                        call.gas.as_usize() as i128,
                        0,
                    ));
                    model.input = Some(call.input.to_vec());
                }
                _ => (),
            };

            match &trace.error {
                Some(_) => (),
                None => {
                    model.output = match &trace.result {
                        Some(result) => match result {
                            Res::Call(res) => {
                                Some(serde_json::to_string(&res).unwrap().into_bytes())
                            }
                            _ => None,
                        },
                        None => None,
                    }
                }
            };

            model
        }
        ActionType::Create => {
            model.r#type = "create".to_string();

            match &trace.action {
                Action::Create(call) => {
                    model.from_address_hash = Some(call.from.as_bytes().to_vec());
                    model.value = Decimal::from_i128_with_scale(call.value.as_usize() as i128, 0);
                    model.gas = Some(Decimal::from_i128_with_scale(
                        call.gas.as_usize() as i128,
                        0,
                    ));
                    model.init = Some(call.init.to_vec());
                }
                _ => (),
            };

            match &trace.error {
                Some(_) => (),
                None => {
                    model.output = match &trace.result {
                        Some(result) => match result {
                            Res::Create(res) => {
                                Some(serde_json::to_string(&res).unwrap().into_bytes())
                            }
                            _ => None,
                        },
                        None => None,
                    }
                }
            };

            model
        }

        ActionType::Suicide => {
            model.r#type = "suicide".to_string();

            match &trace.action {
                Action::Suicide(call) => {
                    model.from_address_hash = Some(call.address.as_bytes().to_vec());
                    model.to_address_hash = Some(call.refund_address.as_bytes().to_vec());
                    model.value = Decimal::from_i128_with_scale(call.balance.as_usize() as i128, 0);
                }
                _ => (),
            };

            model
        }
        ActionType::Reward => model,
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TxCallType {
    #[serde(rename = "call")]
    Call,
    #[serde(rename = "callcode")]
    CallCode,
    #[serde(rename = "delegatecall")]
    DelegateCall,
    #[serde(rename = "staticcall")]
    StaticCall,
}

impl FromStr for TxCallType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "call" => Ok(TxCallType::Call),
            "callcode" => Ok(TxCallType::CallCode),
            "delegatecall" => Ok(TxCallType::DelegateCall),
            "staticcall" => Ok(TxCallType::StaticCall),
            _ => Err(format!("Unknown CallType: {}", s)),
        }
    }
}

impl ToString for TxCallType {
    fn to_string(&self) -> String {
        match self {
            TxCallType::Call => "call".to_string(),
            TxCallType::CallCode => "callcode".to_string(),
            TxCallType::DelegateCall => "delegatecall".to_string(),
            TxCallType::StaticCall => "staticcall".to_string(),
        }
    }
}

#[derive(Debug, PartialEq)]
enum InternalTransactionType {
    Call,
    Create,
    Create2,
    Reward,
    SelfDestruct,
}

impl FromStr for InternalTransactionType {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "call" => Ok(InternalTransactionType::Call),
            "create" => Ok(InternalTransactionType::Create),
            "create2" => Ok(InternalTransactionType::Create2),
            "reward" => Ok(InternalTransactionType::Reward),
            "selfdestruct" => Ok(InternalTransactionType::SelfDestruct),
            _ => Err(()),
        }
    }
}

impl fmt::Display for InternalTransactionType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = match self {
            InternalTransactionType::Call => "call",
            InternalTransactionType::Create => "create",
            InternalTransactionType::Create2 => "create2",
            InternalTransactionType::Reward => "reward",
            InternalTransactionType::SelfDestruct => "selfdestruct",
        };
        write!(f, "{}", s)
    }
}
