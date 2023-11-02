use std::collections::HashMap;

use chrono::Utc;
use entities::address_token_balances::Model as AddressTokenBalanceModel;
use entities::token_transfers::Model as TokenTransferModel;
use entities::tokens::Model as TokenModel;
use ethers::types::H160;

use crate::common::consts;

pub fn process_address_token_balances(
    token_map: HashMap<Vec<u8>, TokenModel>,
    token_transfers: Vec<TokenTransferModel>,
) -> Vec<AddressTokenBalanceModel> {
    let mut resp = vec![];
    for token in token_transfers.iter() {
        let token_type = token_map
            .get(&token.token_contract_address_hash)
            .unwrap()
            .r#type;

        if filter_burn_address(&token_type, token.to_address_hash) {
            continue;
        }

        let mut from_model = AddressTokenBalanceModel {
            address_hash: token.from_address_hash,
            block_number: token.block_number.unwrap_or(0),
            token_contract_address_hash: token.token_contract_address_hash,
            value: None,
            value_fetched_at: None,
            inserted_at: Utc::now().naive_utc(),
            updated_at: Utc::now().naive_utc(),
            token_id: None,
            token_type: Some(token_type),
            id: 0,
        };

        let mut to_model = from_model.clone();
        to_model.address_hash = token.to_address_hash;

        if let Some(token_ids) = token.token_ids {
            for token_id in token_ids.iter() {
                let mut from_model_tmp = from_model.clone();
                from_model_tmp.token_id = Some(*token_id);
                resp.push(from_model_tmp);

                let mut to_model_tmp = to_model.clone();
                to_model_tmp.token_id = Some(*token_id);
                resp.push(to_model_tmp);
            }
            continue;
        }

        if let Some(token_id) = token.token_id {
            from_model.token_id = Some(token_id);
            to_model.token_id = Some(token_id);
        }

        resp.push(from_model);
        resp.push(to_model);
    }

    resp
}

fn filter_burn_address(token_type: &str, to_address: Vec<u8>) -> bool {
    let zero_address: H160 = consts::ZERO_ADDRESS.parse().unwrap();
    to_address == zero_address.as_bytes().to_vec() || token_type != consts::ERC721
}
