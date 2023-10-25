use entities::logs::Model;
use repo::dal::event::Query as DbQuery;

use super::*;
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LogResp {
    pub data: String,
    pub index: i32,
    pub r#type: Option<String>,
    pub topics: Vec<Option<String>>,
    pub address_hash: Option<String>,
    pub transaction_hash: String,
    pub block_hash: String,
    pub block_number: Option<i32>,
}

fn conv_model_to_resp(models: Vec<Model>) -> Vec<LogResp> {
    let mut log_resp_list = vec![];
    for model in models.into_iter() {
        let log = LogResp {
            data: format!("0x{}", hex::encode(model.data.clone())),
            index: model.index,
            r#type: model.r#type,
            topics: vec![
                model.first_topic,
                model.second_topic,
                model.third_topic,
                model.fourth_topic,
            ],
            address_hash: model
                .address_hash
                .map(|addr| format!("0x{}", hex::encode(addr))),
            transaction_hash: format!("0x{}", hex::encode(model.transaction_hash.clone())),
            block_hash: format!("0x{}", hex::encode(model.block_hash.clone())),
            block_number: model.block_number,
        };

        log_resp_list.push(log);
    }

    log_resp_list
}

pub async fn get_transaction_logs(
    Extension(state): Extension<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Json<BaseResponse<Vec<LogResp>>>, AppError> {
    let conn = get_conn(&state);

    if id.len() != 66 || !(id.starts_with("0x") || id.starts_with("0X")) {
        return Err(AppError::from(CoreError::Param(id)));
    }

    let hash = Vec::from_hex(&id[2..id.len()]).map_err(AppError::from)?;
    let res = DbQuery::find_by_tx_hash(conn, hash)
        .await
        .map_err(AppError::from)?;

    Ok(Json(BaseResponse::success(conv_model_to_resp(res))))
}
