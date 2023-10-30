use crate::{common::consts, contracts::erc20::IERC20Call};
use anyhow::{anyhow, Error};
use bigdecimal::FromPrimitive;
use config::db::DB;
use repo::dal::token::{Mutation, Query};
use repo::orm::conn::connect_db;
use sea_orm::{prelude::Decimal, DbConn};
use std::sync::Arc;
use std::time::Duration;
use tokio::time::interval;

// TODO all metadata ok than update skip metadata
// TODO and now, catalog will be set ture at once call metadata
pub async fn handle_erc20_metadata(rpc_url: &str, conn: &DbConn) -> Result<(), Error> {
    let erc20_call = IERC20Call::new(rpc_url);
    match Query::filter_uncataloged(conn, consts::ERC20).await {
        Ok(models) => {
            for mut model in models.into_iter() {
                let contract_addr = format!("0x{}", hex::encode(&model.contract_address_hash));
                if let Ok((name, symbol, decimals, total_supply)) =
                    erc20_call.metadata(contract_addr.as_str()).await
                {
                    model.name = Some(name);
                    model.symbol = Some(symbol);
                    model.decimals = Some(Decimal::new(decimals as i64, 0));
                    model.total_supply = Decimal::from_i128(total_supply.as_u128() as i128);
                    model.cataloged = Some(true);
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

pub async fn strat_token_metadata_task(rpc_url: String, db_cfg: DB) {
    let conn = connect_db(db_cfg).await.unwrap();
    let mut interval = interval(Duration::from_secs(3));
    loop {
        interval.tick().await;
        match handle_erc20_metadata(rpc_url.as_str(), &conn).await {
            Ok(_) => (),
            Err(err) => tracing::error!(message = "token metadata task", err = ?err),
        };
    }
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
                let contract_addr = format!("0x{}", hex::encode(&model.contract_address_hash));
                if let Ok(total_supply) = erc20_call.total_supply(contract_addr.as_str()).await {
                    model.total_supply = Decimal::from_i128(total_supply.as_u128() as i128);
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
