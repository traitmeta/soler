use std::collections::HashMap;
use std::sync::Arc;

use anyhow::{anyhow, Result};
use ethers::{
    prelude::abigen,
    providers::{Http, Provider},
    types::{Address, H160, U256},
};

use common::consts;

abigen!(
    TokenBalance,
    r#"[
        function balanceOf(address account) external view returns (uint256)
    ]"#,
);

abigen!(
    Erc1155TokenBalance,
    r#"[
        function balanceOf(address _owner, uint256 _id) external view returns (uint256)
    ]"#,
);

pub struct TokenBalanceRequest {
    pub token_hash: String,
    pub address_hash: String,
    pub block_number: Option<u64>,
    pub token_id: Option<String>,
    pub token_type: consts::TokenKind,
}

pub struct BalanceReader {
    provider: Provider<Http>,
}

impl BalanceReader {
    pub fn new(rpc_url: &str) -> Self {
        let provider = Provider::<Http>::try_from(rpc_url).unwrap();

        BalanceReader { provider }
    }

    pub async fn get_balances_of(
        &self,
        token_balance_requests: Vec<TokenBalanceRequest>,
        _abi: Vec<HashMap<String, serde_json::Value>>,
    ) -> Vec<Result<U256>> {
        let mut resp = vec![];

        for req in token_balance_requests {
            let res = self.token_balance_call_contract(req).await;
            resp.push(res);
        }

        resp
    }

    pub async fn token_balance_call_contract(
        &self,
        req: TokenBalanceRequest,
    ) -> anyhow::Result<U256> {
        let client = Arc::new(&self.provider);
        let contract_address: Address = req.token_hash.parse().unwrap();
        match req.token_type {
            consts::TokenKind::ERC20 => {
                let contract = TokenBalance::new(contract_address, client);
                let user: H160 = req.address_hash.as_str().parse().unwrap();
                let balance_of_call = contract.balance_of(user);
                let balance_of_call = match req.block_number {
                    Some(num) => balance_of_call.block(num),
                    None => balance_of_call,
                };
                match balance_of_call.call().await {
                    Ok(balance) => Ok(balance),
                    Err(err) => Err(anyhow!(
                        "Erc20 get balance: contract_addr:{}, user:{}, err:{}",
                        req.token_hash,
                        req.address_hash,
                        err.to_string()
                    )),
                }
            }
            consts::TokenKind::ERC1155 => {
                let contract = Erc1155TokenBalance::new(contract_address, client);
                let user: H160 = req.address_hash.as_str().parse().unwrap();
                let token_id: U256 = req.token_id.unwrap().as_str().parse().unwrap();
                let balance_of_call = contract.balance_of(user, token_id);
                let balance_of_call = match req.block_number {
                    Some(num) => balance_of_call.block(num),
                    None => balance_of_call,
                };
                match balance_of_call.call().await {
                    Ok(balance) => Ok(balance),
                    Err(err) => Err(anyhow!(
                        "Erc1155 get balance: contract_addr:{}, user:{}, err:{}",
                        req.token_hash,
                        req.address_hash,
                        err.to_string()
                    )),
                }
            }
            _ => Err(anyhow!("Invalid token type")),
        }
    }
}

#[cfg(test)]
mod tests {
    use common::consts;

    use super::{BalanceReader, TokenBalanceRequest};

    #[test]
    fn test_total_supply() {
        let rt = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap();
        let contract = BalanceReader::new("https://eth.llamarpc.com");
        let req: TokenBalanceRequest = TokenBalanceRequest {
            token_hash: "0x53894EC021245adb6a7C556Bb0F0aD83544C0e33".to_string(),
            address_hash: "0xeA400aF528338401EF19494DA6010DCefdb09804".to_string(),
            block_number: Some(19002882),
            token_id: Some("1".to_string()),
            token_type: consts::TokenKind::ERC1155,
        };
        match rt.block_on(contract.token_balance_call_contract(req)) {
            Ok(total_supply) => {
                assert!(!total_supply.is_zero());
                println!("{}", total_supply.to_string());
            }
            Err(err) => {
                assert!(false);
                println!("{}", err.to_string());
            }
        };

        let req: TokenBalanceRequest = TokenBalanceRequest {
            token_hash: "0xB8c77482e45F1F44dE1745F52C74426C631bDD52".to_string(),
            address_hash: "0x480234599362dC7a76cd99D09738A626F6d77e5F".to_string(),
            block_number: Some(19002882),
            token_id: None,
            token_type: consts::TokenKind::ERC20,
        };
        match rt.block_on(contract.token_balance_call_contract(req)) {
            Ok(total_supply) => {
                assert!(!total_supply.is_zero());
                println!("{}", total_supply.to_string());
            }
            Err(err) => {
                assert!(false);
                println!("{}", err.to_string());
            }
        };
    }
}
