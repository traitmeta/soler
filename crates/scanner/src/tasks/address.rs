use crate::contracts::erc20::IERC20Call;
use std::{str::FromStr, sync::Arc, time::Duration};

use anyhow::{anyhow, Error};
use bigdecimal::BigDecimal;
use chrono::Utc;
use common::chain_ident;
use repo::dal::token_balance::{Mutation, Query};
use sea_orm::DatabaseConnection;
use sea_orm::DbConn;

use tokio::time::interval;

pub async fn handle_address_token_balance(
    erc20_call: &IERC20Call,
    conn: &DbConn,
) -> Result<(), Error> {
    let Ok(models) = Query::unfetched_token_balances(conn).await else {
        return Err(anyhow!(
            "handle_erc20_metadata: unfetched_token_balances failed"
        ));
    };
    for mut model in models.into_iter() {
        let contract_addr = chain_ident!(&model.token_contract_address_hash);
        let address = chain_ident!(&model.address_hash);
        // this is the first try
        match erc20_call
            .balance_of(
                contract_addr.as_str(),
                address.as_str(),
                Some(model.block_number as u64),
            )
            .await
        {
            Ok(balance) => {
                model.value = Some(BigDecimal::from_str(balance.to_string().as_str()).unwrap());
                model.value_fetched_at = Some(Utc::now().naive_utc());
            }
            Err(_) => {}
        }

        tracing::info!(
            "update address token balance id: {}, address: {:?}",
            contract_addr.clone(),
            address.clone(),
        );
        if let Err(e) = Mutation::update_balance(conn, &model).await {
            return Err(anyhow!("Handler Erc20 metadata: {:?}", e.to_string()));
        }
    }
    Ok(())
}

pub fn token_metadata_task(erc20_call: Arc<IERC20Call>, conn: Arc<DatabaseConnection>) {
    tokio::task::spawn(async move {
        let mut interval = interval(Duration::from_secs(300));
        loop {
            interval.tick().await;
            match handle_address_token_balance(erc20_call.as_ref(), conn.as_ref()).await {
                Ok(_) => (),
                Err(err) => tracing::error!(message = "token metadata task", err = ?err),
            };
        }
    });
}
