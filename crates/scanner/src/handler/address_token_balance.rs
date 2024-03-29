use std::collections::HashMap;

use chrono::Utc;
use entities::address_current_token_balances::Model as CurrentTokenBalanceModel;
use entities::address_token_balances::Model as AddressTokenBalanceModel;
use entities::token_transfers::Model as TokenTransferModel;
use entities::tokens::Model as TokenModel;
use ethers::types::H160;

use common::{chain_ident, consts};

pub fn process_token_balances(
    token_map: &HashMap<Vec<u8>, TokenModel>,
    token_transfers: &[TokenTransferModel],
) -> (Vec<AddressTokenBalanceModel>, Vec<CurrentTokenBalanceModel>) {
    let address_token_balance = process_address_token_balances(token_map, token_transfers);
    let current_token_balance = process_current_token_balances(token_map, token_transfers);

    (address_token_balance, current_token_balance)
}

pub fn process_address_token_balances(
    token_map: &HashMap<Vec<u8>, TokenModel>,
    token_transfers: &[TokenTransferModel],
) -> Vec<AddressTokenBalanceModel> {
    let mut resp = vec![];
    for token in token_transfers.iter() {
        let token_type = &token_map
            .get(&token.token_contract_address_hash)
            .unwrap()
            .r#type;
        let mut from_model = AddressTokenBalanceModel {
            address_hash: token.from_address_hash.clone(),
            block_number: token.block_number.unwrap_or(0),
            token_contract_address_hash: token.token_contract_address_hash.clone(),
            value: None,
            value_fetched_at: None,
            inserted_at: Utc::now().naive_utc(),
            updated_at: Utc::now().naive_utc(),
            token_id: None,
            token_type: Some(token_type.to_string()),
            id: 0,
        };

        let mut to_model = from_model.clone();
        to_model.address_hash = token.to_address_hash.clone();
        if let Some(token_ids) = &token.token_ids {
            for token_id in token_ids {
                let mut from_model_tmp = from_model.clone();
                from_model_tmp.token_id = Some(token_id.clone());
                resp.push(from_model_tmp);

                let mut to_model_tmp = to_model.clone();
                to_model_tmp.token_id = Some(token_id.clone());
                resp.push(to_model_tmp);
            }
            continue;
        }

        if let Some(token_id) = token.token_id.as_ref() {
            from_model.token_id = Some(token_id.clone());
            to_model.token_id = Some(token_id.clone());
        }

        if chain_ident!(&from_model.address_hash) != chain_ident!(H160::zero().as_bytes()) {
            resp.push(from_model);
        }

        if !is_erc721_burn(token_type, token.to_address_hash.clone())
            && chain_ident!(&to_model.address_hash) != chain_ident!(H160::zero().as_bytes())
        {
            resp.push(to_model);
        }
    }

    resp
}

fn is_erc721_burn(token_type: &str, to_address: Vec<u8>) -> bool {
    let zero_address: H160 = consts::ZERO_ADDRESS.parse().unwrap();
    if to_address == zero_address.as_bytes().to_vec() && token_type == consts::ERC721 {
        return true;
    }

    false
}

pub fn process_current_token_balances(
    token_map: &HashMap<Vec<u8>, TokenModel>,
    token_transfers: &[TokenTransferModel],
) -> Vec<CurrentTokenBalanceModel> {
    let mut resp = vec![];
    for token in token_transfers.iter() {
        let token_type = &token_map
            .get(&token.token_contract_address_hash)
            .unwrap()
            .r#type;

        if is_erc721_burn(token_type, token.to_address_hash.clone()) {
            continue;
        }

        let mut from_model = CurrentTokenBalanceModel {
            address_hash: token.from_address_hash.clone(),
            block_number: token.block_number.unwrap_or(0),
            token_contract_address_hash: token.token_contract_address_hash.clone(),
            value: None,
            value_fetched_at: None,
            inserted_at: Utc::now().naive_utc(),
            updated_at: Utc::now().naive_utc(),
            token_id: None,
            token_type: Some(token_type.to_string()),
            id: 0,
            old_value: None,
        };

        let mut to_model = from_model.clone();
        to_model.address_hash = token.to_address_hash.clone();

        if let Some(token_ids) = &token.token_ids {
            for token_id in token_ids {
                let mut from_model_tmp = from_model.clone();
                from_model_tmp.token_id = Some(token_id.clone());
                resp.push(from_model_tmp);

                let mut to_model_tmp = to_model.clone();
                to_model_tmp.token_id = Some(token_id.clone());
                resp.push(to_model_tmp);
            }
            continue;
        }

        if let Some(token_id) = token.token_id.as_ref() {
            from_model.token_id = Some(token_id.clone());
            to_model.token_id = Some(token_id.clone());
        }

        resp.push(from_model);
        resp.push(to_model);
    }

    resp
}

#[cfg(test)]
mod tests {
    use common::consts;
    use hex::FromHex;

    use crate::handler::address_token_balance::is_erc721_burn;

    #[test]
    fn test_is_erc721_burn() {
        assert_eq!(
            is_erc721_burn(
                consts::ERC721,
                Vec::from_hex("0000000000000000000000000000000000000000").unwrap()
            ),
            true
        );

        assert_eq!(
            is_erc721_burn(
                consts::ERC721,
                Vec::from_hex("0000000000000000000000000000000000000001").unwrap()
            ),
            false
        );
    }
}
