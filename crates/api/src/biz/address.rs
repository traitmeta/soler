use axum::extract::Query;
use common::consts;
use entities::{address_token_balances::Model as TokenBalanceModel, tokens::Model as TokenModel};
use repo::dal::token_balance::Query as TokenBalanceQuery;
use sea_orm::prelude::Decimal;

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
    pub circulating_market_cap: Option<String>,
    pub decimals: Option<String>,
    pub exchange_rate: Option<String>,
    pub holders: Option<String>,
    pub icon_url: Option<String>,
    pub name: Option<String>,
    pub symbol: Option<String>,
    pub total_supply: Option<String>,
    pub r#type: String,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct TokenInstanceResp {
    pub animation_url: Option<String>,
    pub external_app_url: Option<String>,
    pub id: Option<String>,
    pub image_url: Option<String>,
    pub is_unique: Option<String>,
    pub metadata: Option<String>,
    pub owner: Option<String>,
    pub token: Option<TokenResp>,
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
    Path(id): Path<String>,
    Query(params): Query<TokenBalanceQueryParams>,
) -> Result<Json<BaseResponse<Vec<AddressTokenResp>>>, AppError> {
    let conn = get_conn(&state);

    if id.len() != 66 || !(id.starts_with("0x") || id.starts_with("0X")) {
        return Err(AppError::from(CoreError::Param(id)));
    }

    let hash = Vec::from_hex(&id[2..id.len()]).map_err(AppError::from)?;
    let res = TokenBalanceQuery::find_by_type_with_relation(conn, hash, params.r#type)
        .await
        .map_err(AppError::from)?;

    let mut resp = vec![];

    for (balance, token) in res.iter() {
        match token {
            Some(t) => resp.push(conv_model_to_resp(balance, t)),
            None => (),
        }
    }

    Ok(Json(BaseResponse::success(resp)))
}

fn conv_model_to_resp(model: &TokenBalanceModel, token: &TokenModel) -> AddressTokenResp {
    let mut resp = AddressTokenResp {
        token: TokenResp {
            address: format!(
                "0x{}",
                hex::encode(model.token_contract_address_hash.clone())
            ),
            circulating_market_cap: token.circulating_market_cap.map(|f| f.to_string()),
            decimals: token.decimals.map(|f| f.to_string()),
            exchange_rate: None,
            holders: token.holder_count.map(|f| f.to_string()),
            icon_url: token.icon_url.clone(),
            name: token.name.clone(),
            symbol: token.symbol.clone(),
            total_supply: token.total_supply.map(|f| f.to_string()),
            r#type: model.token_type.clone().unwrap(),
        },
        token_id: None,
        token_instance: None,
        value: model.value.clone().map(|f| f.to_string()),
    };

    let erc1155 = consts::ERC1155.to_string();
    match &model.token_type {
        Some(_erc1155) if *_erc1155 == erc1155 => {
            resp.token_id = model.token_id.clone().map(|f| f.to_string());
            // TODO need query token instance
            resp.token_instance = Some(TokenInstanceResp::default())
        }
        Some(_) => (),
        None => (),
    }
    resp
}
