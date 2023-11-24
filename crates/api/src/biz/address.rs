use axum::extract::Query;
use entities::{
    address_token_balances::Model as TokenBalanceModel, blocks, tokens::Model as TokenModel,
    transactions::Model,
};
use repo::dal::{token::Query as TokenQuery, transaction::Query as DbQuery};
use sea_orm::prelude::{BigDecimal, Decimal};

use super::{
    token_transfer::{decode_token_transfers, TokenTransferResp},
    *,
};

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

fn conv_model_to_resp(
    model: &Model,
    block: Option<blocks::Model>,
    token_transfers: Vec<TokenTransferModel>,
    token_map: HashMap<Vec<u8>, TokenModel>,
) -> TransactionResp {
    let mut resp = TransactionResp {
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
        value: model.value.clone(),
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
        token_transfers: vec![],
    };

    resp.token_transfers = decode_token_transfers(token_map, &token_transfers);

    resp
}

#[derive(Debug, Clone, Deserialize)]
pub struct TokenBalanceQueryParams {
    pub address: String,
    pub types: String,
    pub page_size: Option<u64>,
    pub page: Option<u64>,
}

pub async fn get_address_tokens(
    Extension(state): Extension<Arc<AppState>>,
    Query(params): Query<TokenBalanceQueryParams>,
) -> Result<Json<BaseResponse<AddressTokenResp>>, AppError> {
    let conn = get_conn(&state);

    if params.address.len() != 66
        || !(params.address.starts_with("0x") || params.address.starts_with("0X"))
    {
        return Err(AppError::from(CoreError::Param(params.address)));
    }

    let hash = Vec::from_hex(&params.address[2..params.address.len()]).map_err(AppError::from)?;
    let res = DbQuery::find_by_hash_with_relation(conn, hash)
        .await
        .map_err(AppError::from)?;

    match res {
        Some((tx, block, token_transfers)) => {
            tracing::info!(message = "transaction related block",block = ?block);
            tracing::info!(message = "transaction related token transfers",token_transfers = ?token_transfers);
            let mut token_contracts = vec![];
            for token in token_transfers.iter() {
                token_contracts.push(token.token_contract_address_hash.clone());
            }

            let tokens = TokenQuery::find_by_contract_addresses(conn, token_contracts)
                .await
                .map_err(AppError::from)?;

            let tokens_map = tokens
                .iter()
                .map(|t| (t.contract_address_hash.clone(), t.clone()))
                .collect::<HashMap<Vec<u8>, TokenModel>>();

            Ok(Json(BaseResponse::success(conv_model_to_resp(
                &tx,
                block,
                token_transfers,
                tokens_map,
            ))))
        }
        None => Err(AppError::from(CoreError::NotFound)),
    }
}
