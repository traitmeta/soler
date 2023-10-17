use super::{
    err::{AppError, CoreError},
    response::BaseResponse,
    state::{get_conn, AppState},
    Json,
};
use axum::{extract::Path, Extension};
use chrono::NaiveDateTime;
use entities::{blocks, transactions::Model};
use hex::FromHex;
use repo::dal::transaction::Query as DbQuery;
use sea_orm::prelude::Decimal;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/*
Base Fee = block: base_fee_per_gas
Gas Usage by Txn = Tx: gas_used
Burnt Fee = Base Fee * Gas Usage by Txn
Tx Savings Fees = Max Fee * Gas Usage by Txn - (Base Fee + Max Priority Fee) * Gas Usage by Txn
Transaction fee = Gas Usage by Txn *  Gas Price
*/
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TransactionResp {
    pub cumulative_gas_used: Option<Decimal>,
    pub error: Option<String>,
    pub gas_limit: Decimal,
    pub gas_price: Option<Decimal>,
    pub gas_used: Option<Decimal>,
    pub hash: String,
    pub index: Option<i32>,
    pub input: String,
    pub nonce: i32,
    pub status: Option<i32>,
    pub value: Decimal,
    pub block_time: NaiveDateTime,
    pub block_hash: Option<String>,
    pub block_number: Option<i32>,
    pub from_address_hash: String,
    pub to_address_hash: Option<String>,
    pub created_contract_address_hash: Option<String>,
    pub created_contract_code_indexed_at: Option<NaiveDateTime>,
    pub revert_reason: Option<String>,
    pub max_priority_fee_per_gas: Option<Decimal>,
    pub max_fee_per_gas: Option<Decimal>,
    pub r#type: Option<i32>,
    pub has_error_in_internal_txs: Option<bool>,
}

pub struct LogResp {
    pub data: String,
    pub index: i32,
    pub r#type: Option<String>,
    pub first_topic: Option<String>,
    pub second_topic: Option<String>,
    pub third_topic: Option<String>,
    pub fourth_topic: Option<String>,
    pub inserted_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub address_hash: Option<String>,
    pub transaction_hash: String,
    pub block_hash: String,
    pub block_number: Option<i32>,
}

fn conv_model_to_resp(model: &Model, block: Option<blocks::Model>) -> TransactionResp {
    TransactionResp {
        cumulative_gas_used: model.cumulative_gas_used,
        error: model.error.to_owned(),
        gas_limit: model.gas,
        gas_price: model.gas_price,
        gas_used: model.gas_used,
        hash: format!("0x{}", hex::encode(model.hash.clone())),
        index: model.index,
        input: format!("0x{}", hex::encode(model.input.clone())),
        nonce: model.nonce,
        status: model.status,
        value: model.value,
        block_time: match block {
            Some(b) => b.timestamp,
            None => model.inserted_at,
        },
        block_hash: model
            .block_hash
            .as_ref()
            .map(|hash| format!("0x{}", hex::encode(hash))),
        block_number: model.block_number,
        from_address_hash: format!("0x{}", hex::encode(model.from_address_hash.clone())),
        to_address_hash: model
            .to_address_hash
            .as_ref()
            .map(|to| format!("0x{}", hex::encode(to))),
        created_contract_address_hash: model
            .created_contract_address_hash
            .as_ref()
            .map(|contract_addr| format!("0x{}", hex::encode(contract_addr))),
        created_contract_code_indexed_at: model.created_contract_code_indexed_at,
        revert_reason: model.revert_reason.clone(),
        max_priority_fee_per_gas: model.max_priority_fee_per_gas,
        max_fee_per_gas: model.max_fee_per_gas,
        r#type: model.r#type,
        has_error_in_internal_txs: model.has_error_in_internal_txs,
    }
}

pub async fn get_transaction(
    Extension(state): Extension<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Json<BaseResponse<TransactionResp>>, AppError> {
    let conn = get_conn(&state);

    if id.len() != 66 || !(id.starts_with("0x") || id.starts_with("0X")) {
        return Err(AppError::from(CoreError::Param(id)));
    }

    let hash = Vec::from_hex(&id[2..id.len()]).map_err(AppError::from)?;
    let res = DbQuery::find_by_hash_with_relation(conn, hash)
        .await
        .map_err(AppError::from)?;

    match res {
        Some((tx, block, events)) => {
            tracing::info!(message = "transaction related block",block = ?block);
            tracing::info!(message = "transaction related events",events = ?events);
            Ok(Json(BaseResponse::success(conv_model_to_resp(&tx, block))))
        }
        None => Err(AppError::from(CoreError::NotFound)),
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct QueryParams {
    pub block_height: i64,
    pub page_size: Option<u64>,
    pub page: Option<u64>,
}

pub async fn gets_transaction(
    Extension(state): Extension<Arc<AppState>>,
    Json(payload): Json<QueryParams>,
) -> Result<Json<BaseResponse<Vec<TransactionResp>>>, AppError> {
    let conn = get_conn(&state);
    let res = DbQuery::find_in_page_block(
        conn,
        Some(payload.block_height),
        payload.page,
        payload.page_size,
    )
    .await
    .map_err(AppError::from)?;

    let mut resp = vec![];
    for model in res.0.iter() {
        resp.push(conv_model_to_resp(model, None));
    }

    Ok(Json(BaseResponse::success(resp)))
}