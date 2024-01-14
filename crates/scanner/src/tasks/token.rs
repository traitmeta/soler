use crate::contracts::erc20::IERC20Call;
use crate::evms::eth::EthCli;
use std::{str::FromStr, sync::Arc, time::Duration};

use anyhow::{anyhow, Error};
use bigdecimal::BigDecimal;
use common::chain_ident;
use repo::dal::token::{Mutation, Query};
use sea_orm::DatabaseConnection;
use sea_orm::{prelude::Decimal, DbConn};

use tokio::time::interval;

// all metadata failed then update skip metadata
// or catalog will be set ture
pub async fn handle_metadata(erc20_call: &IERC20Call, conn: &DbConn) -> Result<(), Error> {
    let Ok(models) = Query::filter_uncataloged_and_no_skip_metadata(conn, None).await else {
        return Err(anyhow!("handle_metadata: filter_uncataloged failed"));
    };
    for mut model in models.into_iter() {
        let contract_addr = chain_ident!(&model.contract_address_hash);
        // this is the first try
        match erc20_call.metadata(contract_addr.as_str()).await {
            Ok((name, symbol, decimals, total_supply)) => {
                model.name = Some(name);
                model.symbol = Some(symbol);
                model.decimals = Some(Decimal::new(decimals as i64, 0));
                model.total_supply =
                    Some(BigDecimal::from_str(total_supply.to_string().as_str()).unwrap());
                model.cataloged = Some(true);
            }
            Err(_) => {
                // this is the second try
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
                match erc20_call.total_supply(contract_addr.as_str(), None).await {
                    Ok(total_supply) => {
                        model.total_supply =
                            Some(BigDecimal::from_str(total_supply.to_string().as_str()).unwrap());
                    }
                    Err(_) => err_count += 1,
                };
                model.cataloged = Some(true);
                if err_count >= 4 {
                    model.skip_metadata = Some(true);
                }
            }
        }

        tracing::info!(
            "update metadata id: {}, name: {:?}, symbol: {:?}, decimals: {:?}, total_supply: {:?}",
            contract_addr.clone(),
            &model.name.clone(),
            &model.symbol.clone(),
            &model.decimals,
            &model.total_supply.clone(),
        );
        if let Err(e) = Mutation::update_metadata(conn, &model).await {
            return Err(anyhow!("Handler metadata: {:?}", e.to_string()));
        }
    }
    Ok(())
}

pub fn token_metadata_task(erc20_call: Arc<IERC20Call>, conn: Arc<DatabaseConnection>) {
    tokio::task::spawn(async move {
        let mut interval = interval(Duration::from_secs(300));
        loop {
            interval.tick().await;
            match handle_metadata(erc20_call.as_ref(), conn.as_ref()).await {
                Ok(_) => (),
                Err(err) => tracing::error!(message = "token metadata task", err = ?err),
            };
        }
    });
}

// TODO use channel to receive contranct transfer action and then update contract's total supply
pub fn token_total_updater_task(cli: Arc<EthCli>, erc20_call: Arc<IERC20Call>, conn: Arc<DbConn>) {
    tokio::task::spawn(async move {
        let mut interval = interval(Duration::from_secs(300));
        loop {
            interval.tick().await;
            match handle_token_total_supply(cli.clone(), erc20_call.clone(), conn.clone()).await {
                Ok(_) => (),
                Err(err) => tracing::error!(message = "token total supply task", err = ?err),
            };
        }
    });
}

pub async fn handle_token_total_supply(
    cli: Arc<EthCli>,
    erc20_call: Arc<IERC20Call>,
    conn: Arc<DbConn>,
) -> Result<(), Error> {
    let block_number = cli.get_block_number().await;
    match Query::filter_not_skip_metadata(conn.as_ref(), block_number as i64, None).await {
        Ok(models) => {
            for mut model in models.into_iter() {
                let contract_addr = chain_ident!(&model.contract_address_hash);
                if let Ok(total_supply) = erc20_call
                    .total_supply(contract_addr.as_str(), Some(block_number))
                    .await
                {
                    model.total_supply =
                        Some(BigDecimal::from_str(total_supply.to_string().as_str()).unwrap());
                    model.total_supply_updated_at_block = Some(block_number as i64);
                }
                tracing::info!(
                    "update total_supply contract_address: {:?}, total_supply: {:?}, block_height: {}",
                    contract_addr.clone(),
                    &model.total_supply.clone(),
                    block_number,
                );
                if let Err(e) = Mutation::update_total_supply(conn.as_ref(), &model).await {
                    return Err(anyhow!("Handler total supply: {:?}", e.to_string()));
                }
            }
            Ok(())
        }
        Err(e) => Err(anyhow!("Handler total supply: {:?}", e.to_string())),
    }
}
