use crate::{common::consts, contracts::erc20::IERC20Call};
use anyhow::{anyhow, Error};
use repo::dal::token::{Mutation, Query};
use sea_orm::{prelude::Decimal, DbConn};
use std::collections::HashMap;

pub struct TokenHandler {
    rpc_url: String,
}

impl TokenHandler {
    pub fn new(rpc_url: &str) -> TokenHandler {
        Self {
            rpc_url: rpc_url.to_string(),
        }
    }

    pub async fn handle_erc20_metadata(&self, conn: &DbConn) -> Result<(), Error> {
        let erc20_call = IERC20Call::new(self.rpc_url.as_str());
        match Query::filter_not_skip_metadata(conn, consts::ERC20).await {
            Ok(models) => {
                for mut model in models.into_iter() {
                    let contract_addr = std::str::from_utf8(&model.contract_address_hash).unwrap();
                    if let Ok((name, symbol, decimals)) = erc20_call.metadata(contract_addr).await {
                        model.name = Some(name);
                        model.symbol = Some(symbol);
                        model.decimals = Some(Decimal::new(decimals as i64, 0));
                    }

                    if let Err(e) = Mutation::update_metadata(conn, &model).await {
                        return Err(anyhow!("Handler Erc20 metadata: {:?}", e.to_string()));
                    }
                }
                Ok(())
            }
            Err(e) => Err(anyhow!("Handler Erc20 metadata: {:?}", e.to_string())),
        }
    }
}



mod indexer {
    mod fetcher {
        mod internal_transaction {
            use std::error::Error;
            use std::fmt;

            struct Token {
                // token fields
            }

            struct Address {
                // address fields
            }

            struct Chain {
                // chain fields
            }

            struct MetadataRetriever {
                // metadata retriever fields
            }

            struct BufferedTask {
                // buffered task fields
            }

            struct Tracer {
                // tracer fields
            }

            impl BufferedTask {
                fn buffer(module: &str, token_contract_addresses: Vec<Address>) -> Result<(), Box<dyn Error>> {
                    // buffer implementation
                    Ok(())
                }
            }

            impl Chain {
                fn stream_uncataloged_token_contract_address_hashes(
                    initial_acc: &str,
                    reducer: fn(&str, &str) -> &str,
                    flag: bool,
                ) -> Result<(&str, &str), Box<dyn Error>> {
                    // stream uncataloged token contract address hashes implementation
                    Ok((initial_acc, initial_acc))
                }

                fn token_from_address_hash(
                    token_contract_address: &str,
                    options: HashMap<&str, &str>,
                ) -> Result<(&str, Token), Box<dyn Error>> {
                    // token from address hash implementation
                    Ok((token_contract_address, Token {}))
                }

                fn update_token(token: Token, token_params: HashMap<&str, &str>) -> Result<(), Box<dyn Error>> {
                    // update token implementation
                    Ok(())
                }
            }

            impl BufferedTask {
                fn init(
                    initial_acc: &str,
                    reducer: fn(&str, &str) -> &str,
                    _: &str,
                ) -> Result<(&str, &str), Box<dyn Error>> {
                    // init implementation
                    Ok((initial_acc, initial_acc))
                }

                fn run(
                    token_contract_address: &str,
                    _json_rpc_named_arguments: &str,
                ) -> Result<(), Box<dyn Error>> {
                    // run implementation
                    Ok(())
                }
            }

            impl BufferedTask {
                fn async_fetch(token_contract_addresses: Vec<Address>) -> Result<(), Box<dyn Error>> {
                    // async fetch implementation
                    Ok(())
                }
            }

            impl BufferedTask {
                fn defaults() -> HashMap<&str, usize> {
                    // defaults implementation
                    HashMap::new()
                }
            }
        }
    }
}
