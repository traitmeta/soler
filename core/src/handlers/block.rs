use super::{
    err::{AppError, CoreError},
    response::BaseResponse,
    state::{get_conn, AppState},
    Json,
};
use axum::{extract::Path, Extension};
use entities::blocks::Model;
use hex::FromHex;
use repo::dal::block::Query as DbQuery;
use std::sync::Arc;

pub async fn get_block(
    Extension(state): Extension<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Json<BaseResponse<Model>>, AppError> {
    let conn = get_conn(&state);
    let mut block = None;

    if id.starts_with("0x") || id.starts_with("0X") {
        let hash = Vec::from_hex(&id[2..id.len()]).map_err(AppError::from)?;
        block = DbQuery::find_by_hash(conn, hash)
            .await
            .map_err(AppError::from)?;
    } else {
        let height: i64 = id.parse().map_err(AppError::from)?;
        block = DbQuery::find_by_height(conn, height)
            .await
            .map_err(AppError::from)?;
    }

    match block {
        Some(block) => return Ok(Json(BaseResponse::success(block))),
        None => {
            return Err(AppError::from(CoreError::NotFound));
        }
    }
}
