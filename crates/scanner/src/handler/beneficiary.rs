fn fetch_beneficiaries(
    blocks: Vec<Block>,
    all_transactions: Vec<Transaction>,
    json_rpc_named_arguments: JsonRpcNamedArguments,
) -> FetchedBeneficiaries {
    match Application::get_env("indexer", "fetch_rewards_way") {
        Some("manual") => fetch_beneficiaries_manual(blocks, all_transactions),
        _ => fetch_beneficiaries_by_trace_block(blocks, json_rpc_named_arguments),
    }
}

fn fetch_beneficiaries_manual(
    blocks: Vec<Block>,
    all_transactions: Vec<Transaction>,
) -> FetchedBeneficiaries {
    let block_transactions_map = all_transactions
        .into_iter()
        .group_by(|transaction| transaction.block_number);

    blocks
        .into_iter()
        .map(|block| {
            fetch_beneficiaries_manual(
                block,
                block_transactions_map
                    .get(&block.number)
                    .cloned()
                    .unwrap_or_else(Vec::new),
            )
        })
        .fold(FetchedBeneficiaries::default(), |acc, params_set| {
            FetchedBeneficiaries {
                params_set: acc.params_set.union(&params_set).cloned().collect(),
                ..acc
            }
        })
}

fn fetch_beneficiaries_manual(
    block: Block,
    transactions: Vec<Transaction>,
) -> FetchedBeneficiaries {
    let reward_parts = Chain::block_reward_by_parts(block, transactions);
    reward_parts_to_beneficiaries(reward_parts)
}

fn reward_parts_to_beneficiaries(reward_parts: RewardParts) -> FetchedBeneficiaries {
    let reward = reward_parts
        .static_reward
        .sum(reward_parts.txn_fees)
        .sub(reward_parts.burned_fees)
        .sum(reward_parts.uncle_reward);

    let beneficiary = Beneficiary {
        address_hash: reward_parts.miner_hash,
        block_hash: reward_parts.block_hash,
        block_number: reward_parts.block_number,
        reward,
        address_type: AddressType::Validator,
    };

    FetchedBeneficiaries {
        params_set: vec![beneficiary].into_iter().collect(),
        ..FetchedBeneficiaries::default()
    }
}

fn fetch_beneficiaries_by_trace_block(
    blocks: Vec<Block>,
    json_rpc_named_arguments: JsonRpcNamedArguments,
) -> FetchedBeneficiaries {
    let hash_string_by_number: HashMap<u64, String> = blocks
        .into_iter()
        .filter_map(|block| {
            if let (Some(number), Some(hash_string)) = (block.number, block.hash) {
                Some((number, hash_string))
            } else {
                None
            }
        })
        .collect();

    let block_numbers: Vec<u64> = hash_string_by_number.keys().cloned().collect();

    match EthereumJSONRPC::fetch_beneficiaries(json_rpc_named_arguments, block_numbers) {
        Ok(fetched_beneficiaries) => {
            let consensus_params_set =
                consensus_params_set(fetched_beneficiaries.params_set, &hash_string_by_number);

            FetchedBeneficiaries {
                params_set: consensus_params_set,
                ..fetched_beneficiaries
            }
        }
        Err(reason) => {
            log_error(format!("Could not fetch beneficiaries: {:?}", reason));

            let error = match reason {
                Some(Error { code, message }) => Error { code, message },
                _ => Error {
                    code: -1,
                    message: format!("{:?}", reason),
                },
            };

            let errors: Vec<Error> = hash_string_by_number
                .into_iter()
                .filter_map(|(number, _)| {
                    if let Some(block_number) = number {
                        Some(Error {
                            data: Some(Data { block_number }),
                            ..error.clone()
                        })
                    } else {
                        None
                    }
                })
                .collect();

            FetchedBeneficiaries {
                errors,
                ..FetchedBeneficiaries::default()
            }
        }
    }
}

fn consensus_params_set(
    params_set: HashSet<Beneficiary>,
    hash_string_by_number: &HashMap<u64, String>,
) -> HashSet<Beneficiary> {
    params_set
        .into_iter()
        .filter(|beneficiary| {
            if let Some(block_hash_string) = hash_string_by_number.get(&beneficiary.block_number) {
                if block_hash_string == &beneficiary.block_hash {
                    return true;
                } else {
                    log_debug(format!("fetch beneficiaries reported block number ({}) maps to different ({}) block hash than the one from getBlock ({}). A reorg has occurred.", beneficiary.block_number, block_hash_string, beneficiary.block_hash));
                }
            }
            false
        })
        .collect()
}

fn block_reward_by_parts(block: Block, transactions: Vec<Transaction>) -> BlockReward {
    let Block {
        hash: block_hash,
        number: block_number,
        base_fee_per_gas,
        uncles,
    } = block;

    let txn_fees = txn_fees(&transactions);

    let static_reward = match EmissionReward::find_by_block_range(block_number) {
        Some(emission_reward) => emission_reward.reward,
        None => Wei {
            value: Decimal::new(0),
        },
    };

    let has_uncles = uncles.is_some() && !uncles.unwrap().is_empty();

    let burned_fees = burned_fees(&transactions, base_fee_per_gas);
    let uncle_reward = if has_uncles {
        Wei::mult(static_reward, Decimal::from_float(1.0 / 32.0))
    } else {
        Wei {
            value: Decimal::new(0),
        }
    };

    BlockReward {
        block_number,
        block_hash,
        miner_hash: block.miner_hash,
        static_reward,
        txn_fees: Wei { value: txn_fees },
        burned_fees: burned_fees.unwrap_or(Wei {
            value: Decimal::new(0),
        }),
        uncle_reward: uncle_reward.unwrap_or(Wei {
            value: Decimal::new(0),
        }),
    }
}

fn txn_fees(transactions: Vec<Transaction>) -> Decimal {
    transactions
        .iter()
        .fold(Decimal::new(0), |acc, transaction| {
            let gas_used = Decimal::new(transaction.gas_used);
            let gas_price = Decimal::new(transaction.gas_price);
            let fee = gas_used * gas_price;
            acc + fee
        })
}

fn burned_fees(transactions: Vec<Transaction>, base_fee_per_gas: Option<Decimal>) -> Option<Wei> {
    let burned_fee_counter = transactions
        .iter()
        .fold(Decimal::new(0), |acc, transaction| {
            let gas_used = Decimal::new(transaction.gas_used);
            Decimal::add(acc, gas_used)
        });

    match base_fee_per_gas {
        Some(base_fee) => Some(Wei::mult(
            base_fee_per_gas_to_wei(base_fee),
            burned_fee_counter,
        )),
        None => None,
    }
}
