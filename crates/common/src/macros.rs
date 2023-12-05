#[macro_export]
macro_rules! chain_ident {
    ($x:expr) => {
        format!("0x{}", hex::encode($x))
    };
}

#[cfg(test)]
mod tests {
    use ethers::core::utils::hex::decode as hex_decode;

    #[test]
    fn test_gen_iter() {
        let val_hex_str = "00000000000000000000000000000000000000000000003635c9adc5dea00000";
        let vec1 = hex_decode(val_hex_str).unwrap();
        let hash = chain_ident!(vec1);
        assert_eq!(
            hash,
            "0x00000000000000000000000000000000000000000000003635c9adc5dea00000"
        )
    }
}
