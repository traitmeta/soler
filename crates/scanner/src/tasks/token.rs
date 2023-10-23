use crate::{common::consts, contracts::erc20::IERC20Call};
use anyhow::{anyhow, Error};
use repo::dal::token::{Mutation, Query};
use sea_orm::{prelude::Decimal, DbConn};
use std::time::Duration;
use tokio::time::interval;

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
                    let contract_addr = format!("0x{}", hex::encode(&model.contract_address_hash));
                    if let Ok((name, symbol, decimals)) =
                        erc20_call.metadata(contract_addr.as_str()).await
                    {
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

    pub async fn token_metadata_task(&self, conn: &DbConn) {
        let mut interval = interval(Duration::from_secs(3));
        loop {
            interval.tick().await;
            match self.handle_erc20_metadata(&conn).await {
                Ok(_) => (),
                Err(err) => tracing::error!(message = "token metadata task", err = ?err),
            };
        }
    }
}
