use std::collections::HashSet;
use std::error::Error;
use std::fmt;
use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use tokio::sync::mpsc::{channel, Receiver, Sender};
use tokio::sync::Mutex;
use tokio::time::sleep;

#[derive(Debug)]
struct FetcherError {
    message: String,
}

impl Error for FetcherError {}

impl fmt::Display for FetcherError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

#[derive(Debug)]
struct InternalTransaction {
    block_number: u64,
    hash: String,
    index: u64,
}

#[derive(Debug)]
struct InternalTransactionFetcher {
    max_batch_size: usize,
    max_concurrency: usize,
    timeout: Duration,
    fetcher_channel: Mutex<Sender<Vec<u64>>>,
}

impl InternalTransactionFetcher {
    fn new(max_batch_size: usize, max_concurrency: usize, timeout: Duration) -> Self {
        let (tx, _) = channel(max_concurrency);
        InternalTransactionFetcher {
            max_batch_size,
            max_concurrency,
            timeout,
            fetcher_channel: Mutex::new(tx),
        }
    }

    async fn async_fetch(&self, block_numbers: Vec<u64>) -> Result<(), Box<dyn Error>> {
        let mut unique_numbers: HashSet<u64> = HashSet::new();
        for block_number in block_numbers {
            unique_numbers.insert(block_number);
        }

        let filtered_unique_numbers: Vec<u64> = unique_numbers
            .iter()
            .filter(|&&block_number| block_number > 0)
            .cloned()
            .collect();

        let filtered_unique_numbers_count = filtered_unique_numbers.len();
        println!("fetching internal transactions for blocks");

        let (variant_tx, variant_rx) = channel(1);
        variant_tx.send(EthereumJSONRPC::Nethermind).await?;

        let internal_transactions_params = match variant_rx.recv().await {
            Some(variant) => match variant {
                EthereumJSONRPC::Nethermind
                | EthereumJSONRPC::Erigon
                | EthereumJSONRPC::Besu => {
                    EthereumJSONRPC::fetch_block_internal_transactions(
                        filtered_unique_numbers.clone(),
                        json_rpc_named_arguments,
                    )
                }
                _ => {
                    fetch_block_internal_transactions_by_transactions(
                        filtered_unique_numbers.clone(),
                        json_rpc_named_arguments,
                    )
                }
            },
            None => Err(Box::new(FetcherError {
                message: "Failed to receive variant".to_string(),
            })),
        }?;

        match internal_transactions_params {
            Ok(internal_transactions_params) => {
                safe_import_internal_transaction(internal_transactions_params, filtered_unique_numbers)
            }
            Err(reason) => {
                println!(
                    "failed to fetch internal transactions for blocks: {}",
                    reason.to_string()
                );
                handle_not_found_transaction(reason);
                Ok(())
            }
        }
    }

    async fn fetch_block_internal_transactions_by_transactions(
        &self,
        unique_numbers: Vec<u64>,
        json_rpc_named_arguments: JsonRpcNamedArguments,
    ) -> Result<Vec<InternalTransaction>, Box<dyn Error>> {
        let mut acc_list: Vec<InternalTransaction> = Vec::new();
        for block_number in unique_numbers {
            let transactions = Chain.get_transactions_of_block_number(block_number)?;
            let internal_transactions = EthereumJSONRPC::fetch_internal_transactions(
                transactions,
                json_rpc_named_arguments,
            )?;
            acc_list.extend(internal_transactions);
        }
        Ok(acc_list)
    }

    async fn safe_import_internal_transaction(
        &self,
        internal_transactions_params: Vec<InternalTransaction>,
        block_numbers: Vec<u64>,
    ) -> Result<(), Box<dyn Error>> {
        import_internal_transaction(internal_transactions_params, block_numbers)?;
        Ok(())
    }

    async fn import_internal_transaction(
        &self,
        internal_transactions_params: Vec<InternalTransaction>,
        unique_numbers: Vec<u64>,
    ) -> Result<(), Box<dyn Error>> {
        let internal_transactions_params_without_failed_creations =
            remove_failed_creations(internal_transactions_params);

        let addresses_params = Addresses::extract_addresses(internal_transactions_params_without_failed_creations);

        let address_hash_to_block_number: HashMap<String, u64> = addresses_params
            .iter()
            .map(|address_param| (address_param.hash.to_lowercase(), address_param.fetched_coin_balance_block_number))
            .collect();

        let empty_block_numbers: Vec<u64> = unique_numbers
            .difference(&internal_transactions_params_without_failed_creations.iter().map(|param| param.block_number).collect())
            .cloned()
            .collect();

        let internal_transactions_and_empty_block_numbers: Vec<InternalTransaction> =
            internal_transactions_params_without_failed_creations
                .iter()
                .chain(empty_block_numbers.iter())
                .cloned()
                .collect();

        let imports = Chain.import(
            addresses_params,
            internal_transactions_and_empty_block_numbers,
            self.timeout,
        )?;

        match imports {
            Ok(imported) => {
                Accounts.drop(imported.addresses);
                Blocks.drop_nonconsensus(imported.remove_consensus_of_missing_transactions_blocks);

                async_import_coin_balances(imported, address_hash_to_block_number)?;
                Ok(())
            }
            Err((step, reason, _changes_so_far)) => {
                println!(
                    "failed to import internal transactions for blocks: {}",
                    reason.to_string()
                );
                handle_unique_key_violation(reason, unique_numbers);
                Ok(())
            }
        }
    }

    async fn remove_failed_creations(
        &self,
        internal_transactions_params: Vec<InternalTransaction>,
    ) -> Vec<InternalTransaction> {
        internal_transactions_params
            .iter()
            .map(|internal_transaction_param| {
                let transaction_index = internal_transaction_param.index;
                let block_number = internal_transaction_param.block_number;

                let failed_parent = internal_transactions_params
                    .iter()
                    .filter(|internal_transactions_param| {
                        internal_transactions_param.block_number == block_number
                            && internal_transactions_param.index == transaction_index
                            && internal_transactions_param.trace_address.is_empty()
                            && !internal_transactions_param.error.is_none()
                    })
                    .next();

                if let Some(failed_parent) = failed_parent {
                    InternalTransaction {
                        created_contract_address_hash: None,
                        created_contract_code: None,
                        gas_used: None,
                        output: None,
                        error: failed_parent.error.clone(),
                        ..internal_transaction_param.clone()
                    }
                } else {
                    internal_transaction_param.clone()
                }
            })
            .collect()
    }

    async fn handle_unique_key_violation(
        &self,
        reason: Box<dyn Error>,
        block_numbers: Vec<u64>,
    ) {
        BlocksRunner.invalidate_consensus_blocks(block_numbers);

        println!(
            "unique_violation on internal transactions import, block numbers: {:?}",
            block_numbers
        );
    }

    async fn handle_foreign_key_violation(
        &self,
        internal_transactions_params: Vec<InternalTransaction>,
        block_numbers: Vec<u64>,
    ) {
        BlocksRunner.invalidate_consensus_blocks(block_numbers);

        let transaction_hashes: HashSet<String> = internal_transactions_params
            .iter()
            .map(|param| param.hash.to_string())
            .collect();

        println!(
            "foreign_key_violation on internal transactions import, foreign transactions hashes: {:?}",
            transaction_hashes
        );
    }

    async fn handle_not_found_transaction(&self, errors: Vec<Error>) {
        for error in errors {
            match error {
                Error::HistoricalBackendError(data) => invalidate_block_from_error(data),
                Error::GenesisNotTraceable(data) => invalidate_block_from_error(data),
                Error::TransactionNotFound(data) => invalidate_block_from_error(data),
                _ => (),
            }
        }
    }

    async fn invalidate_block_from_error(&self, data: HashMap<String, String>) {
        if let Some(block_number) = data.get("blockNumber") {
            BlocksRunner.invalidate_consensus_blocks(vec![block_number.parse().unwrap()]);
        }
    }
}

#[async_trait]
impl BufferedTask for InternalTransactionFetcher {
    async fn init(&self, initial: Vec<u64>, reducer: fn(u64, Vec<u64>) -> Vec<u64>) -> Vec<u64> {
        let final_result = Chain.stream_blocks_with_unfetched_internal_transactions(initial, reducer).await;
        final_result
    }

    async fn run(&self, block_numbers: Vec<u64>) -> Result<(), Box<dyn Error>> {
        let unique_numbers: HashSet<u64> = block_numbers.into_iter().collect();
        let filtered_unique_numbers: Vec<u64> = unique_numbers.into_iter().filter(|&block_number| block_number > 0).collect();
        let filtered_unique_numbers_count = filtered_unique_numbers.len();
        println!("fetching internal transactions for blocks");

        let (variant_tx, variant_rx) = channel(1);
        variant_tx.send(EthereumJSONRPC::Nethermind).await?;

        let internal_transactions_params = match variant_rx.recv().await {
            Some(variant) => match variant {
                EthereumJSONRPC::Nethermind
                | EthereumJSONRPC::Erigon
                | EthereumJSONRPC::Besu => {
                    EthereumJSONRPC::fetch_block_internal_transactions(
                        filtered_unique_numbers.clone(),
                        json_rpc_named_arguments,
                    )
                }
                _ => {
                    fetch_block_internal_transactions_by_transactions(
                        filtered_unique_numbers.clone(),
                        json_rpc_named_arguments,
                    )
                }
            },
            None => Err(Box::new(FetcherError {
                message: "Failed to receive variant".to_string(),
            })),
        }?;

        match internal_transactions_params {
            Ok(internal_transactions_params) => {
                safe_import_internal_transaction(internal_transactions_params, filtered_unique_numbers)
            }
            Err(reason) => {
                println!(
                    "failed to fetch internal transactions for blocks: {}",
                    reason.to_string()
                );
                handle_not_found_transaction(reason);
                Ok(())
            }
        }
    }
}

#[tokio::main]
async fn main() {
    let fetcher = InternalTransactionFetcher::new(10, 4, Duration::from_secs(5));
    let block_numbers = vec![1, 2, 3, 4, 5];
    fetcher.async_fetch(block_numbers).await.unwrap();
}
