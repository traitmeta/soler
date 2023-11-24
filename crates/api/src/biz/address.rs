use axum::extract::Query;
use entities::{
    address_token_balances::Model as TokenBalanceModel, blocks, tokens::Model as TokenModel,
    transactions::Model,
};
use repo::dal::{token_balance::Query as TokenBalanceQuery, transaction::Query as DbQuery};
use sea_orm::prelude::{BigDecimal, Decimal};

use super::*;
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AddressTokenResp {
    pub token: TokenResp,
    pub token_id: Option<String>,
    pub token_instance: Option<TokenInstanceResp>,
    pub value: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TokenResp {
    pub address: String,
    pub circulating_market_cap: Option<Decimal>,
    pub decimals: Option<String>,
    pub exchange_rate: Option<String>,
    pub holders: Option<String>,
    pub icon_url: Option<String>,
    pub name: Option<String>,
    pub symbol: Option<String>,
    pub total_supply: Option<String>,
    pub r#type: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TokenInstanceResp {
    pub animation_url: Option<String>,
    pub external_app_url: Option<String>,
    pub id: String,
    pub image_url: Option<String>,
    pub is_unique: Option<String>,
    pub metadata: Option<String>,
    pub owner: Option<String>,
    pub token: TokenResp,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TokenBalanceQueryParams {
    pub address: String,
    pub r#type: String,
    pub page_size: Option<u64>,
    pub page: Option<u64>,
}

pub async fn get_address_tokens(
    Extension(state): Extension<Arc<AppState>>,
    Query(params): Query<TokenBalanceQueryParams>,
) -> Result<Json<BaseResponse<Vec<AddressTokenResp>>>, AppError> {
    let conn = get_conn(&state);

    if params.address.len() != 66
        || !(params.address.starts_with("0x") || params.address.starts_with("0X"))
    {
        return Err(AppError::from(CoreError::Param(params.address)));
    }

    let hash = Vec::from_hex(&params.address[2..params.address.len()]).map_err(AppError::from)?;
    let res = TokenBalanceQuery::finds_by_type(conn, hash, params.r#type)
        .await
        .map_err(AppError::from)?;

    let resp = vec![];

    for model in res.iter() {}

    Ok(Json(BaseResponse::success(resp)))
}

pub async fn get_address_tokens_copy(
    Extension(state): Extension<Arc<AppState>>,
    Path(id): Path<String>,
    Query(params): Query<TokenBalanceQueryParams>,
) -> Result<Json<BaseResponse<Vec<AddressTokenResp>>>, AppError> {
    let conn = get_conn(&state);

    if id.len() != 66 || !(id.starts_with("0x") || id.starts_with("0X")) {
        return Err(AppError::from(CoreError::Param(id)));
    }

    let hash = Vec::from_hex(&id[2..id.len()]).map_err(AppError::from)?;
    let res = TokenBalanceQuery::finds_by_type(conn, hash, params.r#type)
        .await
        .map_err(AppError::from)?;

    let resp = vec![];

    for model in res.iter() {}

    Ok(Json(BaseResponse::success(resp)))
}
