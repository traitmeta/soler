use anyhow::{anyhow, Error};
use ethers::{
    prelude::abigen,
    providers::{Http, Provider},
    types::{Address, H160, U256},
};

use std::sync::Arc;

pub struct IERC20Call {
    provider: Provider<Http>,
}

abigen!(
    IERC20,
    r#"[
        function name() external view returns (string)
        function symbol() external view returns (string)
        function decimals() external view returns (uint8)
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

impl IERC20Call {
    pub fn new(rpc_url: &str) -> IERC20Call {
        let provider = Provider::<Http>::try_from(rpc_url).unwrap();
        Self { provider }
    }

    pub async fn total_supply(&self, contract_address: &str) -> Result<ethers::types::U256, Error> {
        let client = Arc::new(&self.provider);
        let address: Address = contract_address.parse().unwrap();
        let contract = IERC20::new(address, client);
        match contract.total_supply().call().await {
            Ok(total_supply) => Ok(total_supply),
            Err(err) => Err(anyhow!("Erc20 get total supply: {}", err.to_string())),
        }
    }

    pub async fn balance_of(
        &self,
        contract_address: &str,
        user_address: &str,
    ) -> Result<ethers::types::U256, Error> {
        let client = Arc::new(&self.provider);
        let address: Address = contract_address.parse().unwrap();
        let contract = IERC20::new(address, client);
        let user: H160 = user_address.parse().unwrap();
        match contract.balance_of(user).call().await {
            Ok(balance) => Ok(balance),
            Err(err) => Err(anyhow!("Erc20 get user balance: {}", err.to_string())),
        }
    }

    pub async fn name(&self, contract_address: &str) -> Result<String, Error> {
        let client = Arc::new(&self.provider);
        let address: Address = contract_address.parse().unwrap();
        let contract = IERC20::new(address, client);
        match contract.name().call().await {
            Ok(name) => Ok(name),
            Err(err) => Err(anyhow!("Erc20 get user name: {}", err.to_string())),
        }
    }

    pub async fn symbol(&self, contract_address: &str) -> Result<String, Error> {
        let client = Arc::new(&self.provider);
        let address: Address = contract_address.parse().unwrap();
        let contract = IERC20::new(address, client);
        match contract.symbol().call().await {
            Ok(symbol) => Ok(symbol),
            Err(err) => Err(anyhow!("Erc20 get user symbol: {}", err.to_string())),
        }
    }

    pub async fn decimals(&self, contract_address: &str) -> Result<u8, Error> {
        let client = Arc::new(&self.provider);
        let address: Address = contract_address.parse().unwrap();
        let contract = IERC20::new(address, client);
        match contract.decimals().call().await {
            Ok(decimals) => Ok(decimals),
            Err(err) => Err(anyhow!("Erc20 get user decimals: {}", err.to_string())),
        }
    }

    pub async fn metadata(
        &self,
        contract_address: &str,
    ) -> Result<(String, String, u8, U256), Error> {
        let name = match self.name(contract_address).await {
            Ok(s) => s.to_string(),
            Err(e) => return Err(e),
        };

        let symbol = match self.symbol(contract_address).await {
            Ok(s) => s.to_string(),
            Err(e) => return Err(e),
        };

        let decimals = match self.decimals(contract_address).await {
            Ok(s) => s,
            Err(e) => return Err(e),
        };

        let total_supply = match self.total_supply(contract_address).await {
            Ok(s) => s,
            Err(e) => return Err(e),
        };

        Ok((name, symbol, decimals, total_supply))
    }
}

#[cfg(test)]
mod tests {
    use super::IERC20Call;
    const WETH_ADDRESS: &str = "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2";

    #[test]
    #[ignore]
    fn test_total_supply() {
        let rt = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap();
        let contract = IERC20Call::new("https://eth.llamarpc.com");
        match rt.block_on(contract.total_supply(WETH_ADDRESS)) {
            Ok(total_supply) => {
                assert!(!total_supply.is_zero());
            }
            Err(err) => println!("{}", err.to_string()),
        };

        match rt.block_on(contract.total_supply("0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc3")) {
            Ok(total_supply) => {
                assert!(total_supply.as_u32() != 0);
                println!("{}", total_supply.as_u128())
            }
            Err(_) => assert!(true),
        };
    }

    #[test]
    #[ignore]
    fn test_balance_of() {
        let rt = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap();
        let contract = IERC20Call::new("https://eth.llamarpc.com");
        match rt.block_on(
            contract.balance_of(WETH_ADDRESS, "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc3"),
        ) {
            Ok(balance) => {
                println!("{}", balance.as_usize());
                assert!(balance.is_zero());
            }
            Err(err) => println!("{}", err.to_string()),
        };

        match rt.block_on(
            contract.balance_of(WETH_ADDRESS, "0xb5d85CBf7cB3EE0D56b3bB207D5Fc4B82f43F511"),
        ) {
            Ok(balance) => {
                println!("{}", balance.as_usize());
                assert!(!balance.is_zero());
            }
            Err(err) => println!("{}", err.to_string()),
        };
    }

    #[test]
    #[ignore]
    fn test_metadata() {
        let rt = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap();
        let contract = IERC20Call::new("https://eth.llamarpc.com");
        match rt.block_on(IERC20Call::metadata(&contract, WETH_ADDRESS)) {
            Ok((name, symbols, decimals, total_supply)) => {
                println!(
                    "name={}, symbols={}, decimals={}, total_supply={}",
                    name, symbols, decimals, total_supply
                );
                assert!(name == "Wrapped Ether".to_string(),);
                assert!(symbols == "WETH".to_string());
                assert!(decimals == 18);
                assert!(!total_supply.is_zero())
            }
            Err(err) => println!("{}", err.to_string()),
        };

        let erc721 = "0xEC3a9c7d612E0E0326e70D97c9310A5f57f9Af9E";
        match rt.block_on(IERC20Call::metadata(&contract, erc721)) {
            Ok((name, symbols, decimals, total_supply)) => {
                println!(
                    "name={}, symbols={}, decimals={}, total_supply={}",
                    name, symbols, decimals, total_supply
                );
                // assert!(name == "Wrapped Ether".to_string(),);
                // assert!(symbols == "WETH".to_string());
                // assert!(decimals == 18);
                // assert!(!total_supply.is_zero())
            }
            Err(err) => println!("{}", err.to_string()),
        };
    }
}
