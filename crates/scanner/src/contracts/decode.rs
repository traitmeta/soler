use anyhow::anyhow;
use ethers::abi::{decode, ParamType, Token};
use ethers::types::U256;

pub fn decode_erc20_event_data(data: &[u8]) -> anyhow::Result<U256> {
    let binding = vec![ParamType::Uint(256)];
    let erc20_event_type = binding.as_slice();
    match decode(erc20_event_type, data) {
        Ok(tokens) => {
            if let Some(Token::Uint(t)) = tokens.first() {
                Ok(t.clone())
            } else {
                Err(anyhow!("Erc20 decode value error"))
            }
        }
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

// return is (id: u256,value: u256)
pub fn decode_erc1155_single_event_data(data: &[u8]) -> anyhow::Result<(U256, U256)> {
    let binding = vec![ParamType::Uint(256), ParamType::Uint(256)];
    let event_type = binding.as_slice();
    match decode(event_type, data) {
        Ok(tokens) => {
            let id: U256;
            let value: U256;
            if let Some(Token::Uint(token_id)) = tokens.get(0) {
                id = token_id.clone();
            } else {
                return Err(anyhow!("Erc1155 decode token_id error"));
            }

            if let Some(Token::Uint(val)) = tokens.get(1) {
                value = val.clone();
            } else {
                return Err(anyhow!("Erc1155 decode value error"));
            }
            Ok((id, value))
        }
        Err(e) => Err(anyhow!("Erc1155 decode value : {}", e.to_string())),
    }
}

// return is (ids: Array[u256],value: Array[u256])
pub fn decode_erc1155_batch_event_data(data: &[u8]) -> anyhow::Result<(Vec<U256>, Vec<U256>)> {
    let binding = vec![
        ParamType::Array(Box::new(ParamType::Uint(256))),
        ParamType::Array(Box::new(ParamType::Uint(256))),
    ];

    let event_type = binding.as_slice();
    match decode(event_type, data) {
        Ok(tokens) => {
            let mut ids = vec![];
            let mut values = vec![];
            if let Some(Token::Array(token_ids)) = tokens.get(0) {
                for id in token_ids {
                    if let Token::Uint(id_uint) = id {
                        ids.push(id_uint.clone());
                    } else {
                        return Err(anyhow!("Erc1155 decode token_id error"));
                    }
                }
            } else {
                return Err(anyhow!("Erc1155 decode token_ids error"));
            }

            if let Some(Token::Array(vals)) = tokens.get(1) {
                for v in vals {
                    if let Token::Uint(v_uint) = v {
                        values.push(v_uint.clone());
                    } else {
                        return Err(anyhow!("Erc1155 decode value error"));
                    }
                }
            } else {
                return Err(anyhow!("Erc1155 decode values error"));
            }
            Ok((ids, values))
        }
        Err(e) => Err(anyhow!("Erc1155 decode value : {}", e.to_string())),
    }
}

#[cfg(test)]
mod tests {
    use super::{
        decode_erc1155_single_event_data, decode_erc20_event_data, decode_erc721_event_data,
    };
    use ethers::{
        core::utils::hex::decode as hex_decode,
        types::{H160, H256, U256},
    };

    #[test]
    fn test_decode_erc20() {
        let val_hex_str = "00000000000000000000000000000000000000000000003635c9adc5dea00000";
        let vec1 = hex_decode(val_hex_str).unwrap();
        let data: &[u8] = vec1.as_slice();
        let result = decode_erc20_event_data(data);
        match result {
            Ok(res) => {
                assert!(res == val_hex_str.parse::<U256>().unwrap())
            }
            Err(e) => {
                println!("error: {:?}", e);
                assert!(false)
            }
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
            Err(e) => {
                println!("error: {:?}", e);
                assert!(false)
            }
        }
    }

    #[test]
    fn test_decode_erc1155_single() {
        let token_id = "000000000000000000000000112ec3b862ab061609ef01d308109a6691ee6a2d";
        let value = "000000000000000000000000000000000000000000000000000000000001cbd2";
        let data = format!("{}{}", token_id, value);
        let vec1 = hex_decode(data).unwrap();
        let data: &[u8] = vec1.as_slice();
        let result = decode_erc1155_single_event_data(data);
        match result {
            Ok((id, val)) => {
                assert!(id == token_id.parse::<U256>().unwrap());
                assert!(val == value.parse::<U256>().unwrap());
            }
            Err(e) => {
                println!("error: {:?}", e);
                assert!(false)
            }
        }
    }
    #[test]
    fn test_decode_erc1155_batch() {
        let token_id = "000000000000000000000000112ec3b862ab061609ef01d308109a6691ee6a2d";
        let value = "000000000000000000000000000000000000000000000000000000000001cbd2";
        let data = format!("{}{}", token_id, value);
        let vec1 = hex_decode(data).unwrap();
        let data: &[u8] = vec1.as_slice();
        let result = decode_erc1155_single_event_data(data);
        match result {
            Ok((id, val)) => {
                assert!(id == token_id.parse::<U256>().unwrap());
                assert!(val == value.parse::<U256>().unwrap());
            }
            Err(e) => {
                println!("error: {:?}", e);
                assert!(false)
            }
        }
    }
}
