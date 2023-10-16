use chrono::Utc;
use entities::internal_transactions::Model;
use ethers::types::{Action, ActionType, Res, Trace, H256};
use sea_orm::prelude::Decimal;
use std::fmt;
use std::{collections::HashMap, str::FromStr};

/*
* `block_number` - the `Blocks` `number` that the `transaction` is collated into.
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
* `transaction_index` - the `Transactions` `index` of `transaction` in `block_number`.
* `type` - type of internal transaction
* `value` - value of transferred from `from_address` to `to_address`
* `block` - block in which this internal transaction occurred
* `block_hash` - foreign key for `block`
* `block_index` - the index of this internal transaction inside the `block`
* `pending_block` - `nil` if `block` has all its internal transactions fetched
*/

pub fn handler_inner_transaction(traces: &[Trace]) -> Vec<Model> {
    let classified_trace = classify_txs(traces);
    process_inner_transaction(classified_trace)
}

pub fn classify_txs(internal_transactions: &[Trace]) -> HashMap<H256, Vec<(Trace, i32)>> {
    let mut tx_map: HashMap<H256, Vec<(Trace, i32)>> = HashMap::new();
    for (i, tx) in internal_transactions.iter().enumerate() {
        if let Some(hash) = tx.transaction_hash {
            match tx_map.get_mut(&hash) {
                Some(map) => map.push((tx.clone(), i as i32)),
                None => {
                    tx_map.insert(hash, vec![(tx.clone(), i as i32)]);
                }
            }
        }
    }

    tx_map
}

fn process_inner_transaction(traces: HashMap<H256, Vec<(Trace, i32)>>) -> Vec<Model> {
    let mut res = vec![];
    for (_key, val) in traces.iter() {
        for (idx, item) in val.iter().enumerate() {
            let model = internal_transaction_to_model(item, idx as i32);
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
        transaction_index: trace.transaction_position.map(|pos| pos as i32),
        block_hash: trace.block_hash.as_bytes().to_vec(),
        block_index: *block_idx,
    };

    match trace.action_type {
        ActionType::Call => {
            model.r#type = "call".to_string();

            if let Action::Call(call) = &trace.action {
                let call_typ = serde_json::to_string(&call.call_type).unwrap();
                model.call_type = Some(call_typ);
                model.from_address_hash = Some(call.from.as_bytes().to_vec());
                model.to_address_hash = Some(call.to.as_bytes().to_vec());
                model.gas = Some(Decimal::from_i128_with_scale(
                    call.gas.as_usize() as i128,
                    0,
                ));
                model.input = Some(call.input.to_vec());
            };

            match &trace.error {
                Some(_) => (),
                None => match &trace.result {
                    Some(Res::Call(res)) => {
                        model.gas_used = Some(Decimal::from_i128_with_scale(
                            res.gas_used.as_usize() as i128,
                            0,
                        ));
                        model.output = Some(serde_json::to_string(&res).unwrap().into_bytes())
                    }
                    Some(_) => (),
                    None => (),
                },
            };

            model
        }
        ActionType::Create => {
            model.r#type = "create".to_string();

            if let Action::Create(call) = &trace.action {
                model.from_address_hash = Some(call.from.as_bytes().to_vec());
                model.value = Decimal::from_i128_with_scale(call.value.as_usize() as i128, 0);
                model.gas = Some(Decimal::from_i128_with_scale(
                    call.gas.as_usize() as i128,
                    0,
                ));
                model.init = Some(call.init.to_vec());
            };

            match &trace.error {
                Some(_) => (),
                None => match &trace.result {
                    Some(Res::Create(res)) => {
                        model.created_contract_code = Some(res.code.to_vec());
                        model.created_contract_address_hash = Some(res.address.as_bytes().to_vec());
                        model.gas_used = Some(Decimal::from_i128_with_scale(
                            res.gas_used.as_usize() as i128,
                            0,
                        ));
                    }
                    Some(_) => (),
                    None => (),
                },
            };

            model
        }

        ActionType::Suicide => {
            model.r#type = "suicide".to_string();

            if let Action::Suicide(call) = &trace.action {
                model.from_address_hash = Some(call.address.as_bytes().to_vec());
                model.to_address_hash = Some(call.refund_address.as_bytes().to_vec());
                model.value = Decimal::from_i128_with_scale(call.balance.as_usize() as i128, 0);
            };

            model
        }
        ActionType::Reward => {
            model.r#type = "reward".to_string();

            model
        }
    }
}

#[derive(Debug, PartialEq)]
enum InternalActionType {
    Call,
    Create,
    Create2,
    Reward,
    SelfDestruct,
}

impl FromStr for InternalActionType {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "call" => Ok(InternalActionType::Call),
            "create" => Ok(InternalActionType::Create),
            "create2" => Ok(InternalActionType::Create2),
            "reward" => Ok(InternalActionType::Reward),
            "selfdestruct" => Ok(InternalActionType::SelfDestruct),
            _ => Err(()),
        }
    }
}

impl fmt::Display for InternalActionType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = match self {
            InternalActionType::Call => "call",
            InternalActionType::Create => "create",
            InternalActionType::Create2 => "create2",
            InternalActionType::Reward => "reward",
            InternalActionType::SelfDestruct => "selfdestruct",
        };
        write!(f, "{}", s)
    }
}
