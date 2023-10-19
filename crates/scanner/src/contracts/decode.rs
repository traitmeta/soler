use anyhow::anyhow;
use ethers::abi::{decode, ParamType, Token};
use ethers::types::{H160, U256};

pub fn decode_erc20_event_data(data: &[u8]) -> anyhow::Result<U256> {
    let binding = vec![ParamType::Uint(256)];
    let erc20_event_type = binding.as_slice();
    match decode(erc20_event_type, data) {
        Ok(tokens) => {
            if let Some(Token::Uint(t)) = tokens.first() {
                Ok(*t)
            } else {
                Err(anyhow!("Erc20 decode value error"))
            }
        }
        Err(e) => Err(anyhow!("Erc20 decode value : {}", e.to_string())),
    }
}

// return is (from: address,to: address, value(token_id): u256)
pub fn decode_erc721_event_data(data: &[u8]) -> anyhow::Result<(H160, H160, U256)> {
    let binding = vec![ParamType::Address, ParamType::Address, ParamType::Uint(256)];
    let event_type = binding.as_slice();
    match decode(event_type, data) {
        Ok(tokens) => {
            let from: H160;
            let to: H160;
            let token_id: U256;
            if let Some(Token::Address(addr)) = tokens.get(0) {
                from = *addr;
            } else {
                return Err(anyhow!("Erc721 decode from address error"));
            }

            if let Some(Token::Address(addr)) = tokens.get(1) {
                to = *addr;
            } else {
                return Err(anyhow!("Erc721 decode to address error"));
            }

            if let Some(Token::Uint(token)) = tokens.get(2) {
                token_id = *token;
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
                id = *token_id;
            } else {
                return Err(anyhow!("Erc1155 decode token_id error"));
            }

            if let Some(Token::Uint(val)) = tokens.get(1) {
                value = *val;
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
                        ids.push(*id_uint);
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
                        values.push(*v_uint);
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
    use std::str::FromStr;

    use super::{
        decode_erc1155_batch_event_data, decode_erc1155_single_event_data, decode_erc20_event_data,
        decode_erc721_event_data,
    };
    use ethers::{
        core::utils::hex::decode as hex_decode,
        types::{H160, H256, U256},
    };
    use sea_orm::prelude::{BigDecimal, Decimal};

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
                assert!(from == H160::from(from_addr.parse::<H256>().unwrap()));
                assert!(to == H160::from(to_addr.parse::<H256>().unwrap()));
                assert!(token == token_id.parse::<U256>().unwrap());
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
                let val_dec = BigDecimal::from_str(id.to_string().as_str()).unwrap();
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
        let token_ids_hex = vec![
            "0000000000000000000000000000000000000000000000000000000000012a76",
            "0000000000000000000000000000000000000000000000000000000000012a77",
            "0000000000000000000000000000000000000000000000000000000000012a78",
            "0000000000000000000000000000000000000000000000000000000000012a79",
            "0000000000000000000000000000000000000000000000000000000000012a7a",
            "0000000000000000000000000000000000000000000000000000000000012a7b",
            "0000000000000000000000000000000000000000000000000000000000012a7c",
            "0000000000000000000000000000000000000000000000000000000000012a7d",
            "0000000000000000000000000000000000000000000000000000000000012a7e",
            "0000000000000000000000000000000000000000000000000000000000012a7f",
            "0000000000000000000000000000000000000000000000000000000000012a80",
            "0000000000000000000000000000000000000000000000000000000000012a81",
            "0000000000000000000000000000000000000000000000000000000000012a82",
            "0000000000000000000000000000000000000000000000000000000000012a83",
            "0000000000000000000000000000000000000000000000000000000000012a84",
            "0000000000000000000000000000000000000000000000000000000000012a85",
            "0000000000000000000000000000000000000000000000000000000000012a86",
            "0000000000000000000000000000000000000000000000000000000000012a87",
            "0000000000000000000000000000000000000000000000000000000000012a88",
            "0000000000000000000000000000000000000000000000000000000000012a89",
            "0000000000000000000000000000000000000000000000000000000000012a8a",
            "0000000000000000000000000000000000000000000000000000000000012a8b",
            "0000000000000000000000000000000000000000000000000000000000012a8c",
            "0000000000000000000000000000000000000000000000000000000000012a8d",
            "0000000000000000000000000000000000000000000000000000000000012a8e",
            "0000000000000000000000000000000000000000000000000000000000012a8f",
            "0000000000000000000000000000000000000000000000000000000000012a90",
            "0000000000000000000000000000000000000000000000000000000000012a91",
            "0000000000000000000000000000000000000000000000000000000000012a92",
            "0000000000000000000000000000000000000000000000000000000000012a93",
            "0000000000000000000000000000000000000000000000000000000000012a94",
            "0000000000000000000000000000000000000000000000000000000000012a95",
            "0000000000000000000000000000000000000000000000000000000000012a96",
            "0000000000000000000000000000000000000000000000000000000000012a97",
            "0000000000000000000000000000000000000000000000000000000000012a98",
        ];
        let values_hex = vec![
            "0000000000000000000000000000000000000000000000000000000000000001",
            "0000000000000000000000000000000000000000000000000000000000000001",
            "0000000000000000000000000000000000000000000000000000000000000001",
            "0000000000000000000000000000000000000000000000000000000000000001",
            "0000000000000000000000000000000000000000000000000000000000000001",
            "0000000000000000000000000000000000000000000000000000000000000001",
            "0000000000000000000000000000000000000000000000000000000000000001",
            "0000000000000000000000000000000000000000000000000000000000000001",
            "0000000000000000000000000000000000000000000000000000000000000001",
            "0000000000000000000000000000000000000000000000000000000000000001",
            "0000000000000000000000000000000000000000000000000000000000000001",
            "0000000000000000000000000000000000000000000000000000000000000001",
            "0000000000000000000000000000000000000000000000000000000000000001",
            "0000000000000000000000000000000000000000000000000000000000000001",
            "0000000000000000000000000000000000000000000000000000000000000001",
            "0000000000000000000000000000000000000000000000000000000000000001",
            "0000000000000000000000000000000000000000000000000000000000000001",
            "0000000000000000000000000000000000000000000000000000000000000001",
            "0000000000000000000000000000000000000000000000000000000000000001",
            "0000000000000000000000000000000000000000000000000000000000000001",
            "0000000000000000000000000000000000000000000000000000000000000001",
            "0000000000000000000000000000000000000000000000000000000000000001",
            "0000000000000000000000000000000000000000000000000000000000000001",
            "0000000000000000000000000000000000000000000000000000000000000001",
            "0000000000000000000000000000000000000000000000000000000000000001",
            "0000000000000000000000000000000000000000000000000000000000000001",
            "0000000000000000000000000000000000000000000000000000000000000001",
            "0000000000000000000000000000000000000000000000000000000000000001",
            "0000000000000000000000000000000000000000000000000000000000000001",
            "0000000000000000000000000000000000000000000000000000000000000001",
            "0000000000000000000000000000000000000000000000000000000000000001",
            "0000000000000000000000000000000000000000000000000000000000000001",
            "0000000000000000000000000000000000000000000000000000000000000001",
            "0000000000000000000000000000000000000000000000000000000000000001",
            "0000000000000000000000000000000000000000000000000000000000000001",
        ];
        let vec1 = hex_decode("000000000000000000000000000000000000000000000000000000000000004000000000000000000000000000000000000000000000000000000000000004c000000000000000000000000000000000000000000000000000000000000000230000000000000000000000000000000000000000000000000000000000012a760000000000000000000000000000000000000000000000000000000000012a770000000000000000000000000000000000000000000000000000000000012a780000000000000000000000000000000000000000000000000000000000012a790000000000000000000000000000000000000000000000000000000000012a7a0000000000000000000000000000000000000000000000000000000000012a7b0000000000000000000000000000000000000000000000000000000000012a7c0000000000000000000000000000000000000000000000000000000000012a7d0000000000000000000000000000000000000000000000000000000000012a7e0000000000000000000000000000000000000000000000000000000000012a7f0000000000000000000000000000000000000000000000000000000000012a800000000000000000000000000000000000000000000000000000000000012a810000000000000000000000000000000000000000000000000000000000012a820000000000000000000000000000000000000000000000000000000000012a830000000000000000000000000000000000000000000000000000000000012a840000000000000000000000000000000000000000000000000000000000012a850000000000000000000000000000000000000000000000000000000000012a860000000000000000000000000000000000000000000000000000000000012a870000000000000000000000000000000000000000000000000000000000012a880000000000000000000000000000000000000000000000000000000000012a890000000000000000000000000000000000000000000000000000000000012a8a0000000000000000000000000000000000000000000000000000000000012a8b0000000000000000000000000000000000000000000000000000000000012a8c0000000000000000000000000000000000000000000000000000000000012a8d0000000000000000000000000000000000000000000000000000000000012a8e0000000000000000000000000000000000000000000000000000000000012a8f0000000000000000000000000000000000000000000000000000000000012a900000000000000000000000000000000000000000000000000000000000012a910000000000000000000000000000000000000000000000000000000000012a920000000000000000000000000000000000000000000000000000000000012a930000000000000000000000000000000000000000000000000000000000012a940000000000000000000000000000000000000000000000000000000000012a950000000000000000000000000000000000000000000000000000000000012a960000000000000000000000000000000000000000000000000000000000012a970000000000000000000000000000000000000000000000000000000000012a98000000000000000000000000000000000000000000000000000000000000002300000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000001000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000001000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000001000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000001000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000001000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000001000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000001000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000001000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000001000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000001000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000001000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000001").unwrap();
        let data: &[u8] = vec1.as_slice();
        let result = decode_erc1155_batch_event_data(data);
        match result {
            Ok((ids, vals)) => {
                assert!(ids.len() == token_ids_hex.len());
                assert!(vals.len() == values_hex.len());

                for (idx, id) in ids.iter().enumerate() {
                    assert!(*id == token_ids_hex[idx].parse::<U256>().unwrap());
                }
                for (idx, val) in vals.iter().enumerate() {
                    assert!(*val == values_hex[idx].parse::<U256>().unwrap());
                }
            }
            Err(e) => {
                println!("error: {:?}", e);
                assert!(false)
            }
        }
    }
}
