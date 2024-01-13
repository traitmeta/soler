use crate::contracts::erc20::IERC20Call;

use anyhow::{anyhow, Error};
use bigdecimal::BigDecimal;
use common::{chain_ident, consts};
use repo::dal::token::{Mutation, Query};
use sea_orm::DatabaseConnection;
use sea_orm::{prelude::Decimal, DbConn};
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::interval;

// all metadata ok than update skip metadata
// and now, catalog will be set ture at once call metadata
pub async fn handle_erc20_metadata(erc20_call: &IERC20Call, conn: &DbConn) -> Result<(), Error> {
    match Query::filter_uncataloged(conn, consts::ERC20).await {
        Ok(models) => {
            for mut model in models.into_iter() {
                let contract_addr = chain_ident!(&model.contract_address_hash);
                match erc20_call.metadata(contract_addr.as_str()).await {
                    Ok((name, symbol, decimals, total_supply)) => {
                        model.name = Some(name);
                        model.symbol = Some(symbol);
                        model.decimals = Some(Decimal::new(decimals as i64, 0));
                        model.total_supply =
                            Some(BigDecimal::from_str(total_supply.to_string().as_str()).unwrap());
                        model.skip_metadata = Some(true);
                    }
                    Err(e) => {
                        let mut err_count = 0;
                        match erc20_call.name(contract_addr.as_str()).await {
                            Ok(name) => model.name = Some(name),
                            Err(_) => err_count += 1,
                        };
                        match erc20_call.symbol(contract_addr.as_str()).await {
                            Ok(symbol) => model.symbol = Some(symbol),
                            Err(_) => err_count += 1,
                        };
                        match erc20_call.decimals(contract_addr.as_str()).await {
                            Ok(decimals) => model.decimals = Some(Decimal::new(decimals as i64, 0)),
                            Err(_) => err_count += 1,
                        };
                        match erc20_call.total_supply(contract_addr.as_str()).await {
                            Ok(total_supply) => {
                                model.total_supply = Some(
                                    BigDecimal::from_str(total_supply.to_string().as_str())
                                        .unwrap(),
                                );
                            }
                            Err(_) => err_count += 1,
                        };
                        model.cataloged = Some(true);
                        if err_count >= 4 {
                            return Err(anyhow!(
                                "Handler Erc20 metadata all fail: {:?}",
                                e.to_string()
                            ));
                        }
                    }
                }

                tracing::info!(
                    "update erc20 metadata name: {}, symbol: {}, decimals: {}, total_supply: {}",
                    &model.name.clone().unwrap(),
                    &model.symbol.clone().unwrap(),
                    &model.decimals.unwrap(),
                    &model.total_supply.clone().unwrap(),
                );
                if let Err(e) = Mutation::update_metadata(conn, &model).await {
                    return Err(anyhow!("Handler Erc20 metadata: {:?}", e.to_string()));
                }
            }
            Ok(())
        }
        Err(e) => Err(anyhow!("Handler Erc20 metadata: {:?}", e.to_string())),
    }
}

pub fn strat_token_metadata_task(erc20_call: IERC20Call, conn: Arc<DatabaseConnection>) {
    tokio::task::spawn(async move {
        let mut interval = interval(Duration::from_secs(3));
        loop {
            interval.tick().await;
            match handle_erc20_metadata(&erc20_call, conn.as_ref()).await {
                Ok(_) => (),
                Err(err) => tracing::error!(message = "token metadata task", err = ?err),
            };
        }
    });
}

pub async fn strat_token_total_updater_task(rpc_url: String, conn: Arc<DbConn>) {
    let mut interval = interval(Duration::from_secs(3));
    let erc20_call = Arc::new(IERC20Call::new(rpc_url.as_str()));
    loop {
        interval.tick().await;
        match handle_erc20_total_supply(erc20_call.clone(), conn.clone()).await {
            Ok(_) => (),
            Err(err) => tracing::error!(message = "token total supply task", err = ?err),
        };
    }
}

pub async fn handle_erc20_total_supply(
    erc20_call: Arc<IERC20Call>,
    conn: Arc<DbConn>,
) -> Result<(), Error> {
    match Query::filter_uncataloged(conn.as_ref(), consts::ERC20).await {
        Ok(models) => {
            for mut model in models.into_iter() {
                let contract_addr = chain_ident!(&model.contract_address_hash);
                if let Ok(total_supply) = erc20_call.total_supply(contract_addr.as_str()).await {
                    model.total_supply =
                        Some(BigDecimal::from_str(total_supply.to_string().as_str()).unwrap());
                }

                if let Err(e) = Mutation::update_total_supply(conn.as_ref(), &model).await {
                    return Err(anyhow!("Handler Erc20 total supply: {:?}", e.to_string()));
                }
            }
            Ok(())
        }
        Err(e) => Err(anyhow!("Handler Erc20 total supply: {:?}", e.to_string())),
    }
}
