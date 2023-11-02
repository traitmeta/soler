use entities::{token_transfers::Model, tokens::Model as TokenModel};
use repo::dal::token_transfer::Query as DbQuery;

use super::*;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TokenTransferResp {
    pub transaction_hash: String,
    pub log_index: i32,
    pub from: String,
    pub to: String,
    pub token: token::TokenResp,
    pub block_number: Option<i64>,
    pub block_hash: String,
    pub total: Vec<TotalTokenDetail>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TotalTokenDetail {
    pub amount: Option<String>,
    pub decimals: Option<String>,
    pub token_id: Option<String>,
}

pub fn decode_token_transfers(
    token_map: HashMap<Vec<u8>, TokenModel>,
    token_transfers: &[Model],
) -> Vec<TokenTransferResp> {
    let mut resp = vec![];
    for token in token_transfers.iter() {
        let token_info = token_map.get(&token.token_contract_address_hash).unwrap();
        let mut transfers = decode_token_transfer(token_info, token);
        resp.append(&mut transfers);
    }
    resp
}

pub fn decode_token_transfer(token: &TokenModel, token_transfer: &Model) -> Vec<TokenTransferResp> {
    let mut token_transfers = vec![];
    let mut resp = TokenTransferResp {
        transaction_hash: format!("0x{}", hex::encode(token_transfer.transaction_hash.clone())),
        log_index: token_transfer.log_index,
        from: format!(
            "0x{}",
            hex::encode(token_transfer.from_address_hash.clone())
        ),
        to: format!("0x{}", hex::encode(token_transfer.to_address_hash.clone())),
        token: token::conv_model_to_resp(token),
        block_number: token_transfer.block_number,
        block_hash: format!("0x{}", hex::encode(token_transfer.block_hash.clone())),
        total: vec![],
    };

    if let Some(amount) = &token_transfer.amount {
        let detail = TotalTokenDetail {
            amount: Some(amount.to_string()),
            decimals: token.decimals.map(|d| d.to_string()),
            token_id: None,
        };
        resp.total.push(detail);
    }

    if let Some(token_id) = &token_transfer.token_id {
        let detail = TotalTokenDetail {
            amount: Some("1".to_string()),
            decimals: token.decimals.map(|d| d.to_string()),
            token_id: Some(token_id.to_string()),
        };
        resp.total.push(detail);
    }

    if let (Some(amounts), Some(token_ids)) = (&token_transfer.amounts, &token_transfer.token_ids) {
        assert!(
            amounts.len() == token_ids.len(),
            "amounts len = {}, token_ids len = {}",
            amounts.len(),
            token_ids.len()
        );
        for (idx, amount) in amounts.iter().enumerate() {
            let detail = TotalTokenDetail {
                amount: Some(amount.to_string()),
                decimals: token.decimals.map(|d| d.to_string()),
                token_id: Some(token_ids[idx].to_string()),
            };
            resp.total.push(detail);
        }
    }

    token_transfers.push(resp);
    token_transfers
}

pub async fn get_token_transfers(
    Extension(state): Extension<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Json<BaseResponse<Vec<TokenTransferResp>>>, AppError> {
    let conn = get_conn(&state);

    if id.len() != 66 || !(id.starts_with("0x") || id.starts_with("0X")) {
        return Err(AppError::from(CoreError::Param(id)));
    }

    let hash = Vec::from_hex(&id[2..id.len()]).map_err(AppError::from)?;
    let res = DbQuery::find_by_tx(conn, hash)
        .await
        .map_err(AppError::from)?;

    let mut token_contracts = vec![];
    for token in res.iter() {
        token_contracts.push(token.token_contract_address_hash.clone());
    }

    let tokens = repo::dal::token::Query::find_by_contract_addresses(conn, token_contracts)
        .await
        .map_err(AppError::from)?;

    let tokens_map = tokens
        .iter()
        .map(|t| (t.contract_address_hash.clone(), t.clone()))
        .collect::<HashMap<Vec<u8>, TokenModel>>();

    Ok(Json(BaseResponse::success(decode_token_transfers(
        tokens_map, &res,
    ))))
}
