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
    pub token_contract_address_hash: String,
    pub address_hash: String,
    pub block_number: Option<u64>,
    pub token_id: Option<U256>,
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
        let address: Address = req.token_contract_address_hash.parse().unwrap();
        match req.token_type {
            consts::TokenKind::ERC20 => {
                let contract = TokenBalance::new(address, client);
                let user: H160 = req.address_hash.parse().unwrap();
                let balance_of_call = contract.balance_of(user);
                let balance_of_call = match req.block_number {
                    Some(num) => balance_of_call.block(num),
                    None => balance_of_call,
                };
                match balance_of_call.call().await {
                    Ok(balance) => Ok(balance),
                    Err(err) => Err(anyhow!(
                        "Erc20 get balance: contract_addr:{}, user:{}, err:{}",
                        req.token_contract_address_hash,
                        req.address_hash,
                        err.to_string()
                    )),
                }
            }
            consts::TokenKind::ERC1155 => {
                let contract = Erc1155TokenBalance::new(address, client);
                let user: H160 = req.address_hash.parse().unwrap();
                let balance_of_call = contract.balance_of(user, req.token_id.unwrap());
                let balance_of_call = match req.block_number {
                    Some(num) => balance_of_call.block(num),
                    None => balance_of_call,
                };
                match balance_of_call.call().await {
                    Ok(balance) => Ok(balance),
                    Err(err) => Err(anyhow!(
                        "Erc1155 get balance: contract_addr:{}, user:{}, err:{}",
                        req.token_contract_address_hash,
                        req.address_hash,
                        err.to_string()
                    )),
                }
            }
            _ => Err(anyhow!("Invalid token type")),
        }
    }
}
