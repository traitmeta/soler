use anyhow::anyhow;
use ethers::abi::{decode, ParamType, Token};
use ethers::abi::{AbiDecode, Uint};
use ethers::types::U256;

use crate::handler::token;

pub fn decode_erc20_event_data(data: &[u8]) -> anyhow::Result<Option<U256>> {
    let binding = vec![ParamType::Uint(256)];
    let erc20_event_type = binding.as_slice();
    match decode(erc20_event_type, data) {
        Ok(tokens) => match tokens.first() {
            Some(t) => Ok(t.clone().into_uint()),
            None => Ok(None),
        },
        Err(e) => Err(anyhow!("Erc20 decode value : {}", e.to_string())),
    }
}

// return is (from: address,to: address, value(token_id): u256)
pub fn decode_erc721_event_data(data: &[u8]) -> anyhow::Result<(Vec<u8>, Vec<u8>, Vec<u8>)> {
    let binding = vec![ParamType::Address, ParamType::Address, ParamType::Uint(256)];
    let event_type = binding.as_slice();
    match decode(event_type, data) {
        Ok(tokens) => {
            let from: Vec<u8>;
            let to: Vec<u8>;
            let token_id: Vec<u8>;
            if let Some(Token::Address(addr)) = tokens.get(0) {
                from = addr.as_bytes().to_vec();
            } else {
                return Err(anyhow!("Erc721 decode from address error"));
            }

            if let Some(Token::Address(addr)) = tokens.get(1) {
                to = addr.as_bytes().to_vec();
            } else {
                return Err(anyhow!("Erc721 decode to address error"));
            }

            if let Some(Token::Uint(token)) = tokens.get(2) {
                token_id = token.to_string().into_bytes();
            } else {
                return Err(anyhow!("Erc721 decode token_id error"));
            }

            Ok((from, to, token_id))
        }
        Err(e) => Err(anyhow!("Erc721 decode value : {}", e.to_string())),
    }
}

// TODO return is (u256,u256)
pub fn decode_erc1155_single_event_data(data: &[u8]) -> anyhow::Result<Option<U256>> {
    let binding = vec![ParamType::Uint(256), ParamType::Uint(256)];
    let event_type = binding.as_slice();
    match decode(event_type, data) {
        Ok(tokens) => match tokens.first() {
            Some(t) => Ok(t.clone().into_uint()),
            None => Ok(None),
        },
        Err(e) => Err(anyhow!("Erc20 decode value : {}", e.to_string())),
    }
}

pub fn decode_erc1155_batch_event_data(data: &[u8]) -> anyhow::Result<Option<U256>> {
    let binding = vec![
        ParamType::Array(Box::new(ParamType::Uint(256))),
        ParamType::Array(Box::new(ParamType::Uint(256))),
    ];

    // TODO how to make sure that there is param1 and param2 ?
    let event_type = binding.as_slice();
    match decode(event_type, data) {
        Ok(tokens) => match tokens.first() {
            Some(t) => Ok(t.clone().into_uint()),
            None => Ok(None),
        },
        Err(e) => Err(anyhow!("Erc20 decode value : {}", e.to_string())),
    }
}

#[cfg(test)]
mod tests {
    use crate::handler::token;

    use super::{decode_erc20_event_data, decode_erc721_event_data};
    use ethers::{
        abi::Hash,
        core::utils::hex::decode as hex_decode,
        types::{Address, H160, H256, U256},
    };
    use sea_orm::Related;

    #[test]
    fn test_decode_erc20() {
        let vec1 =
            hex_decode("00000000000000000000000000000000000000000000003635c9adc5dea00000").unwrap();
        let data: &[u8] = vec1.as_slice();
        let result = decode_erc20_event_data(data);
        match result {
            Ok(result) => match result {
                Some(value) => println!("result: {:?}", value),
                None => println!("result is None"),
            },
            Err(e) => println!("error: {:?}", e),
        }

        let vec1 =
            hex_decode("000000000000000000000000000000000000000000000000000000000001cbd2").unwrap();
        let data: &[u8] = vec1.as_slice();
        let result = decode_erc20_event_data(data);
        match result {
            Ok(result) => match result {
                Some(value) => println!("result: {:?}", value),
                None => println!("result is None"),
            },
            Err(e) => println!("error: {:?}", e),
        }
    }

    #[test]
    fn test_decode_erc721() {
        let from_addr = "0000000000000000000000000000000000000000000000000000000000000000";
        let to_addr = "000000000000000000000000112ec3b862ab061609ef01d308109a6691ee6a2d";
        let token_id = "000000000000000000000000000000000000000000000000000000000001cbd2";
        let data = format!("{}{}{}", from_addr, to_addr, token_id);
        let vec1 = hex_decode(data).unwrap();
        let data: &[u8] = vec1.as_slice();
        let result = decode_erc721_event_data(data);
        match result {
            Ok((from, to, token)) => {
                assert!(from == H160::from(from_addr.parse::<H256>().unwrap()).as_bytes());
                assert!(to == H160::from(to_addr.parse::<H256>().unwrap()).as_bytes());
                assert!(token == token_id.parse::<U256>().unwrap().to_string().into_bytes());
            }
            Err(e) => println!("error: {:?}", e),
        }
    }
}
