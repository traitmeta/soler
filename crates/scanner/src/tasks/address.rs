use crate::contracts::balance_reader::BalanceReader;
use crate::indexer::token_balances::fetch_token_balances_from_blockchain;
use std::{sync::Arc, time::Duration};

use anyhow::{anyhow, Error};
use repo::dal::token_balance::{Mutation, Query};
use sea_orm::DatabaseConnection;
use sea_orm::DbConn;

use tokio::time::interval;

pub async fn handle_address_token_balance(
    reader: &BalanceReader,
    conn: &DbConn,
) -> Result<(), Error> {
    let Ok(models) = Query::unfetched_token_balances(conn).await else {
        return Err(anyhow!(
            "handle_erc20_metadata: unfetched_token_balances failed"
        ));
    };

    let res = fetch_token_balances_from_blockchain(reader, models)
        .await
        .unwrap();
    for model in res.into_iter() {
        if let Err(e) = Mutation::update_balance(conn, &model).await {
            return Err(anyhow!(
                "Handler address_token_balance: {:?}",
                e.to_string()
            ));
        }
        tracing::info!("update address token => model: {:?}", &model);
    }
    Ok(())
}

pub fn address_token_balance_task(reader: Arc<BalanceReader>, conn: Arc<DatabaseConnection>) {
    tokio::task::spawn(async move {
        let mut interval = interval(Duration::from_secs(300));
        loop {
            interval.tick().await;
            match handle_address_token_balance(reader.as_ref(), conn.as_ref()).await {
                Ok(_) => (),
                Err(err) => tracing::error!(message = "token metadata task", err = ?err),
            };
        }
    });
}
