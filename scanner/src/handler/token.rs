use crate::{common::consts, contracts::erc20::IERC20Call};
use anyhow::{anyhow, Error};
use repo::dal::token::{Mutation, Query};
use sea_orm::{prelude::Decimal, DbConn};

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
            Err(e) => return Err(anyhow!("Handler Erc20 metadata: {:?}", e.to_string())),
        }
    }
}
