use super::{
    err::{AppError, CoreError},
    response::BaseResponse,
    state::{get_conn, AppState},
    Json,
};
use axum::{extract::Path, Extension};
use entities::transactions::Model;
use hex::FromHex;
use repo::dal::transaction::Query as DbQuery;
use serde::Deserialize;
use std::sync::Arc;

// TODO change vec<u8> to string?
pub async fn get_transaction(
    Extension(state): Extension<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Json<BaseResponse<Model>>, AppError> {
    let conn = get_conn(&state);

    if id.len() != 66 || !(id.starts_with("0x") || id.starts_with("0X")) {
        return Err(AppError::from(CoreError::Param(id)));
    }

    let hash = Vec::from_hex(&id[2..id.len()]).map_err(AppError::from)?;
    let res = DbQuery::find_by_hash(conn, hash)
        .await
        .map_err(AppError::from)?;

    match res {
        Some(res) => Ok(Json(BaseResponse::success(res))),
        None => Err(AppError::from(CoreError::NotFound)),
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct QueryParams {
    pub block_height: i64,
    pub page_size: Option<u64>,
    pub page: Option<u64>,
}

// TODO change vec<u8> to string?
pub async fn gets_transaction(
    Extension(state): Extension<Arc<AppState>>,
    Json(payload): Json<QueryParams>,
) -> Result<Json<BaseResponse<Vec<Model>>>, AppError> {
    let conn = get_conn(&state);
    let res = DbQuery::find_in_page_block(
        conn,
        Some(payload.block_height),
        payload.page,
        payload.page_size,
    )
    .await
    .map_err(AppError::from)?;

    Ok(Json(BaseResponse::success(res.0)))
}
