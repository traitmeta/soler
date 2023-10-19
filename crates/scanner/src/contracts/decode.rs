use anyhow::anyhow;
use ethers::abi::{decode, ParamType, Token};
use ethers::abi::{AbiDecode, Uint};
use ethers::types::U256;

// fn main() {
//     let vec1 =
//         hex_decode("a934861a0000000000000000000000000000000000000000000000000000000000000005")
//             .unwrap();
//     let data: &[u8] = vec1.as_slice();
//     let result = decode(vec![ParamType::Uint(256)].as_slice(), data).unwrap();
//     println!("{:?}", result);
// }

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

#[cfg(test)]
mod tests {
    use super::decode_erc20_event_data;
    use ethers::core::utils::hex::decode as hex_decode;

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
}
