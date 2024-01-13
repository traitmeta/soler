use common::chain_ident;
use entities::tokens::Model;
use serde::{Deserialize, Serialize};

/*
    When process in api, need to rename transfer type
    - mint => to not zero and from is zero
    - burn => from not zero and to is zero
    - create => from and to is zero
    - transfer => from and to is not zero
*/
pub const ZERO_ADDRESS: &str = "0x0000000000000000000000000000000000000000";
pub const TOKEN_BURN: &str = "Token Burning";
pub const TOKEN_MINT: &str = "Token Minting";
pub const TOKEN_TRANSFER: &str = "Token Transfer";
pub const TOKEN_CREATION: &str = "Token Creation";

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TokenResp {
    pub address: String,
    pub circulating_market_cap: Option<String>,
    pub decimals: Option<String>,
    pub holders: Option<String>,
    pub icon_url: Option<String>,
    pub name: Option<String>,
    pub symbol: Option<String>,
    pub total_supply: Option<String>,
    pub r#type: String,
}

pub fn conv_model_to_resp(model: &Model) -> TokenResp {
    TokenResp {
        address: chain_ident!(model.contract_address_hash.clone()),
        circulating_market_cap: model.circulating_market_cap.map(|c| c.to_string()),
        decimals: model.decimals.map(|c| c.to_string()),
        holders: model.holder_count.map(|c| c.to_string()),
        icon_url: model.icon_url.clone(),
        name: model.name.clone(),
        symbol: model.symbol.clone(),
        total_supply: model.total_supply.clone().map(|c| c.to_string()),
        r#type: model.r#type.clone(),
    }
}

pub fn rename_transfer_type(from: Vec<u8>, to: Vec<u8>) -> String {
    if from == ZERO_ADDRESS.as_bytes().to_vec() {
        if to == ZERO_ADDRESS.as_bytes().to_vec() {
            return TOKEN_CREATION.to_string();
        } else {
            return TOKEN_MINT.to_string();
        }
    } else if to == ZERO_ADDRESS.as_bytes().to_vec() {
        return TOKEN_BURN.to_string();
    }

    TOKEN_TRANSFER.to_string()
}
