use std::collections::{HashMap, HashSet};

use tracing::debug;

struct TokenBalance {
    token_contract_address_hash: String,
    address_hash: String,
    block_number: u32,
    token_type: String,
    token_id: u32,
}

struct TokenBalanceFetcher {
    max_retries: u32,
}

impl TokenBalanceFetcher {
    fn new() -> Self {
        TokenBalanceFetcher { max_retries: 3 }
    }

    fn async_fetch(&self, token_balances: Vec<TokenBalance>) {
        if TokenBalanceSupervisor::disabled() {
            return;
        }

        let formatted_params: Vec<HashMap<String, String>> = token_balances
            .iter()
            .map(|balance| self.format_params(balance))
            .collect();

        TokenBalanceFetcher::buffer(formatted_params, std::usize::MAX);
    }

    fn format_params(&self, token_balance: &TokenBalance) -> HashMap<String, String> {
        let mut params = HashMap::new();
        params.insert(
            "token_contract_address_hash".to_string(),
            token_balance.token_contract_address_hash.clone(),
        );
        params.insert(
            "address_hash".to_string(),
            token_balance.address_hash.clone(),
        );
        params.insert(
            "block_number".to_string(),
            token_balance.block_number.to_string(),
        );
        params.insert("token_type".to_string(), token_balance.token_type.clone());
        params.insert("token_id".to_string(), token_balance.token_id.to_string());
        params
    }

    fn fetch_from_blockchain(
        &self,
        params_list: Vec<HashMap<String, String>>,
    ) -> Vec<HashMap<String, String>> {
        let mut retryable_params_list: Vec<HashMap<String, String>> = params_list
            .iter()
            .filter(|params| {
                let retries_count = params.get("retries_count").unwrap().parse::<u32>().unwrap();
                retries_count <= self.max_retries
            })
            .cloned()
            .collect();

        let mut fetched_token_balances = Vec::new();
        let mut failed_token_balances = retryable_params_list.clone();

        for _ in 0..self.max_retries {
            let token_balances =
                TokenBalances::fetch_token_balances_from_blockchain(failed_token_balances.clone());

            if token_balances.failed_token_balances.is_empty() {
                fetched_token_balances = token_balances.fetched_token_balances;
                break;
            } else {
                failed_token_balances = self.increase_retries_count(failed_token_balances);
            }
        }

        fetched_token_balances
    }

    fn increase_retries_count(
        &self,
        params_list: Vec<HashMap<String, String>>,
    ) -> Vec<HashMap<String, String>> {
        params_list
            .iter()
            .map(|params| {
                let mut updated_params = params.clone();
                let retries_count = params.get("retries_count").unwrap().parse::<u32>().unwrap();
                updated_params.insert("retries_count".to_string(), (retries_count + 1).to_string());
                updated_params
            })
            .collect()
    }

    fn import_token_balances(&self, token_balances_params: Vec<HashMap<String, String>>) {
        let addresses_params = self.format_and_filter_address_params(&token_balances_params);
        let formatted_token_balances_params =
            self.format_and_filter_token_balance_params(&token_balances_params);

        let import_params = ImportParams {
            addresses: AddressesParams {
                params: addresses_params,
            },
            address_token_balances: AddressTokenBalancesParams {
                params: formatted_token_balances_params.clone(),
            },
            address_current_token_balances: AddressCurrentTokenBalancesParams {
                params: TokenBalances::to_address_current_token_balances(
                    formatted_token_balances_params,
                ),
            },
            timeout: std::time::Duration::from_secs(u64::MAX),
        };

        match Chain::import(import_params) {
            Ok(_) => (),
            Err(reason) => {
                debug!(
                    "failed to import token balances: reason = {},error_count = {}",
                    reason,
                    token_balances_params.len()
                );
            }
        }
    }

    fn format_and_filter_address_params(
        &self,
        token_balances_params: &Vec<HashMap<String, String>>,
    ) -> Vec<HashMap<String, String>> {
        let mut address_params: Vec<HashMap<String, String>> = Vec::new();
        let mut address_set: HashSet<String> = HashSet::new();

        for params in token_balances_params {
            let address_hash = params.get("address_hash").unwrap().clone();
            if !address_set.contains(&address_hash) {
                let mut address_param = HashMap::new();
                address_param.insert("hash".to_string(), address_hash.clone());
                address_params.push(address_param);
                address_set.insert(address_hash);
            }
        }

        address_params
    }

    fn format_and_filter_token_balance_params(
        &self,
        token_balances_params: &Vec<HashMap<String, String>>,
    ) -> Vec<HashMap<String, String>> {
        token_balances_params
            .iter()
            .map(|params| {
                let mut updated_params = params.clone();
                if !updated_params.contains_key("token_type") {
                    let token_type = Chain::get_token_type(
                        updated_params
                            .get("token_contract_address_hash")
                            .unwrap()
                            .clone(),
                    );
                    if let Some(token_type) = token_type {
                        updated_params.insert("token_type".to_string(), token_type);
                    }
                }
                updated_params
            })
            .collect()
    }
}

struct TokenBalanceSupervisor;

impl TokenBalanceSupervisor {
    fn disabled() -> bool {
        // Implementation for checking if TokenBalanceSupervisor is disabled
        false
    }
}

struct TokenBalances;

impl TokenBalances {
    fn fetch_token_balances_from_blockchain(
        failed_token_balances: Vec<HashMap<String, String>>,
    ) -> TokenBalanceResult {
        // Implementation for fetching token balances from blockchain
        TokenBalanceResult {
            fetched_token_balances: Vec::new(),
            failed_token_balances,
        }
    }

    fn to_address_current_token_balances(
        token_balances_params: Vec<HashMap<String, String>>,
    ) -> Vec<HashMap<String, String>> {
        // Implementation for converting token balances to address current token balances
        Vec::new()
    }
}

struct Chain;

impl Chain {
    fn import(import_params: ImportParams) -> Result<(), String> {
        // Implementation for importing token balances to the database
        Ok(())
    }

    fn get_token_type(token_contract_address_hash: String) -> Option<String> {
        // Implementation for getting token type from token contract address hash
        None
    }
}

struct ImportParams {
    addresses: AddressesParams,
    address_token_balances: AddressTokenBalancesParams,
    address_current_token_balances: AddressCurrentTokenBalancesParams,
    timeout: std::time::Duration,
}

struct AddressesParams {
    params: Vec<HashMap<String, String>>,
}

struct AddressTokenBalancesParams {
    params: Vec<HashMap<String, String>>,
}

struct AddressCurrentTokenBalancesParams {
    params: Vec<HashMap<String, String>>,
}

struct TokenBalanceResult {
    fetched_token_balances: Vec<HashMap<String, String>>,
    failed_token_balances: Vec<HashMap<String, String>>,
}

trait BufferedTask {
    fn buffer(params: Vec<HashMap<String, String>>, max_batch_size: usize);
}

impl BufferedTask for TokenBalanceFetcher {
    fn buffer(params: Vec<HashMap<String, String>>, max_batch_size: usize) {
        // Implementation for buffering token balance params
    }
}

fn main() {
    let token_balances = vec![
        TokenBalance {
            token_contract_address_hash: "token_contract_1".to_string(),
            address_hash: "address_1".to_string(),
            block_number: 1,
            token_type: "token_type_1".to_string(),
            token_id: 1,
        },
        TokenBalance {
            token_contract_address_hash: "token_contract_2".to_string(),
            address_hash: "address_2".to_string(),
            block_number: 2,
            token_type: "token_type_2".to_string(),
            token_id: 2,
        },
    ];

    let fetcher = TokenBalanceFetcher::new();
    fetcher.async_fetch(token_balances);
}
