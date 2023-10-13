
use std::{str::FromStr, collections::HashMap};
use chrono::Utc;
use entities::internal_transactions::Model;
use ethers::types::{Trace, ActionType, Action, CallType};
use sea_orm::prelude::Decimal;
use serde::{Serialize, Deserialize};
use std::fmt;

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


fn classify_txs(internal_transactions: Vec<Trace>) -> HashMap<String, Vec<(Trace,i32)>> {
    let mut tx_map:HashMap<String, Vec<(Trace,i32)>> = HashMap::new();
    for i in 0..internal_transactions.len(){
        let tx = internal_transactions[i];
        if let Some(hash) = tx.transaction_hash{
            match tx_map.get_mut(&hash.to_string()) {
               Some(mut map) => map.push((tx,i as i32)),
               None =>                 {
                tx_map.insert(hash.to_string(), vec![(tx,i as i32)]);
            },
            }
        }
    }

    tx_map
}



fn internal_transactions_to_raw(traces: HashMap<String, Vec<(Trace,i32)>>) -> Vec<Model> {
    let mut res = vec![];

    for (&key, &val) in traces.iter(){
        for idx in 0..val.len(){
            let model = internal_transaction_to_raw(&val[idx],idx as i32);
            res.push(model);
        }
    }

    res
}

fn internal_transaction_to_raw(transaction: &(Trace,i32), idx :i32) -> Model {
    let (trace, block_idx) = transaction;
    let model = Model {
        call_type: None,
        created_contract_code: None,
        error: None,
        gas: None,
        gas_used: None,
        index: idx ,
        init: None,
        input: None,
        output: None,
        trace_address: trace.trace_address.iter().map(|&f | f as i32 ).collect(),
        r#type:  "".to_string(),
        value: Decimal::ZERO,
        inserted_at: Utc::now().naive_utc(),
        updated_at: Utc::now().naive_utc(),
        created_contract_address_hash: None,
        from_address_hash: None,
        to_address_hash: None,
        transaction_hash: match trace.transaction_hash{
            Some(hash)=> hash.as_bytes().to_vec(),
            None=> Vec::new(),
        },
        block_number: Some(trace.block_number as i32),
        transaction_index: match trace.transaction_position{
            Some(position)=> Some(position as i32),
            None=> None,
        },
        block_hash: trace.block_hash.as_bytes().to_vec(),
        block_index: *block_idx,
    };

    match trace.action_type {
        ActionType::Call => {
                model.r#type= "call".to_string();

            match trace.action{
                Action::Call(call) =>{
                    let call_typ = serde_json::to_string(&call.call_type).unwrap();
                    model.call_type= Some(call_typ);
                    model.from_address_hash= Some(call.from.as_bytes().to_vec());
                    model.to_address_hash= Some(call.to.as_bytes().to_vec());
                    model.gas = Some(Decimal::from_i128_with_scale(call.gas.as_usize() as i128, 0));
                    model.input = Some(call.input.to_vec());
                },
                _ => ()
            };

            match trace.error {
                Some(error) => model.error = Some(error),
                None => model.output = match trace.result{
                    Some(result) => match result{
                        Res::Call(res) => res.
                    }
                }
            }
        }
        InternalTransactionType::Create | InternalTransactionType::Create2 => {
            let InternalTransaction::Create {
                from_address_hash,
                gas,
                init,
                trace_address,
                value,
            } = transaction;

            let action = Action {
                from: from_address_hash.clone(),
                gas: gas.clone(),
                init: init.clone(),
                value: value.clone(),
            };

            let raw_transaction = RawTransaction {
                type: transaction.type.to_string(),
                action: Action::to_raw(action),
                trace_address: trace_address.clone(),
            };

            raw_transaction.put_raw_create_error_or_result(transaction)
        }
        InternalTransactionType::SelfDestruct => {
            let InternalTransaction::SelfDestruct {
                to_address_hash,
                from_address_hash,
                trace_address,
                value,
            } = transaction;

            let action = Action {
                address: from_address_hash.clone(),
                balance: value.clone(),
                refund_address: to_address_hash.clone(),
            };

            let raw_transaction = RawTransaction {
                type: "suicide".to_string(),
                action: Action::to_raw(action),
                trace_address: trace_address.clone(),
            };

            raw_transaction
        }
    }
}

fn add_subtraces(traces: Vec<RawTransaction>) -> Vec<RawTransaction> {
    traces
        .iter()
        .map(|trace| {
            let subtraces = count_subtraces(trace, &traces);
            trace.put("subtraces", subtraces)
        })
        .collect()
}

fn count_subtraces(trace: &RawTransaction, traces: &Vec<RawTransaction>) -> usize {
    traces
        .iter()
        .filter(|t| direct_descendant(&trace.trace_address, &t.trace_address))
        .count()
}

fn direct_descendant(trace_address1: &[u8], trace_address2: &[u8]) -> bool {
    if trace_address1.is_empty() && !trace_address2.is_empty() {
        return true;
    }

    if trace_address1.len() > trace_address2.len() {
        return false;
    }

    if trace_address1[0] == trace_address2[0] {
        direct_descendant(&trace_address1[1..], &trace_address2[1..])
    } else {
        false
    }
}

fn put_raw_call_error_or_result(model: &mut Model, trace: &Trace) -> RawTransaction {
    match trace.error {
        Some(error) => model.error = Some(error),
        None => model.result = Some()
            "result",
            Result::to_raw(Result {
                gas_used: transaction.gas_used,
                output: transaction.output.clone(),
            }),
        ),
    }
}

fn put_raw_create_error_or_result(raw: RawTransaction, transaction: &InternalTransaction) -> RawTransaction {
    match transaction.error {
        Some(error) => raw.put("error", error),
        None => raw.put(
            "result",
            Result::to_raw(Result {
                gas_used: transaction.gas_used,
                code: transaction.created_contract_code.clone(),
                address: transaction.created_contract_address_hash.clone(),
            }),
        ),
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


