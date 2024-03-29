use entities::blocks::Model;
use repo::dal::block::Query as DbQuery;
use sea_orm::prelude::Decimal;

use super::*;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BlockResp {
    pub difficulty: Option<Decimal>,
    pub gas_limit: Decimal,
    pub gas_used: Decimal,
    pub hash: String,
    pub miner_hash: String,
    pub nonce: String,
    pub number: i64,
    pub parent_hash: String,
    pub size: Option<i32>,
    pub timestamp: NaiveDateTime,
    pub total_difficulty: Option<Decimal>,
    pub base_fee_per_gas: Option<Decimal>,
    pub total_transaction: u64,
    pub total_withdraw: u64,
}

fn conv_model_to_resp(model: Model) -> BlockResp {
    BlockResp {
        difficulty: model.difficulty,
        gas_limit: model.gas_limit,
        gas_used: model.gas_used,
        hash: chain_ident!(model.hash),
        miner_hash: chain_ident!(model.miner_hash),
        nonce: chain_ident!(model.nonce),
        number: model.number,
        parent_hash: chain_ident!(model.parent_hash),
        size: model.size,
        timestamp: model.timestamp,
        total_difficulty: model.total_difficulty,
        base_fee_per_gas: model.base_fee_per_gas,
        total_transaction: 0, // need graphQL to finish
        total_withdraw: 0,
    }
}

pub async fn get_block(
    Extension(state): Extension<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Json<BaseResponse<BlockResp>>, AppError> {
    let conn = get_conn(&state);
    let block = if id.starts_with("0x") || id.starts_with("0X") {
        let hash = Vec::from_hex(&id[2..id.len()]).map_err(AppError::from)?;
        DbQuery::find_by_hash(conn, hash)
            .await
            .map_err(AppError::from)?
    } else {
        let height: i64 = id.parse().map_err(AppError::from)?;
        DbQuery::find_by_height(conn, height)
            .await
            .map_err(AppError::from)?
    };

    match block {
        Some(block) => Ok(Json(BaseResponse::success(conv_model_to_resp(block)))),
        None => Err(AppError::from(CoreError::NotFound)),
    }
}
