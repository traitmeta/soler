use ethers::abi::decode;
use ethers::abi::ParamType;
use ethers::core::utils::hex::decode as hex_decode;

fn main() {
    let vec1 =
        hex_decode("a934861a0000000000000000000000000000000000000000000000000000000000000005")
            .unwrap();
    let data: &[u8] = vec1.as_slice();
    let result = decode(vec![ParamType::Uint(256)].as_slice(), data).unwrap();
    println!("{:?}", result);
}

fn decode_erc20_event_data(data: &[u8]) -> {
    let erc20_event_type = vec![ParamType::Uint(256)].as_slice();
    decode(vec![ParamType::Uint(256)].as_slice(), data).unwrap()
}
