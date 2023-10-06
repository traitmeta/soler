use ethers::contract::ContractError;
use ethers::{
    prelude::{abigen, Abigen},
    providers::{Http, Provider},
    types::Address,
};
use std::sync::Arc;

const WETH_ADDRESS: &str = "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2";

pub async fn call_contract(
    rpc_url: &str,
    contract_address: &str,
) -> Result<ethers::types::U256, ContractError<Provider<Http>>> {
    // The abigen! macro expands the contract's code in the current scope
    // so that you can interface your Rust program with the blockchain
    // counterpart of the contract.
    abigen!(
        IERC20,
        r#"[
            function totalSupply() external view returns (uint256)
            function balanceOf(address account) external view returns (uint256)
            function transfer(address recipient, uint256 amount) external returns (bool)
            function allowance(address owner, address spender) external view returns (uint256)
            function approve(address spender, uint256 amount) external returns (bool)
            function transferFrom( address sender, address recipient, uint256 amount) external returns (bool)
            event Transfer(address indexed from, address indexed to, uint256 value)
            event Approval(address indexed owner, address indexed spender, uint256 value)
        ]"#,
    );

    let provider = Provider::<Http>::try_from(rpc_url).unwrap();
    let client = Arc::new(provider);
    let address: Address = contract_address.parse().unwrap();
    let contract = IERC20::new(address, client);

    tracing::info!("WETH total supply begin");
    contract.total_supply().call().await
}

#[cfg(test)]
mod tests {
    use super::{call_contract, WETH_ADDRESS};

    #[test]
    #[ignore]
    fn test_total_supply() {
        let rt = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap();

        match rt.block_on(call_contract("https://eth.llamarpc.com", WETH_ADDRESS)) {
            Ok(total_supply) => {
                assert!(!total_supply.is_zero());
            }
            Err(err) => println!("{}", err.to_string()),
        };

        match rt.block_on(call_contract("https://eth.llamarpc.com", "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc3")) {
            Ok(total_supply) => {
                assert!(total_supply.as_u32()!=0);
                println!("{}", total_supply.as_u128())
            }
            Err(_) => assert!(true),
        };
    }
}
