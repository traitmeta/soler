use std::{sync::Arc, time::Duration};

use repo::dal::block::Query;
use sea_orm::DatabaseConnection;

use tokio::time::interval;

use crate::evms::eth::EthCli;
use crate::handler::block::{handle_block, sync_to_db};

pub fn handle_block_task(cli: Arc<EthCli>, conn: Arc<DatabaseConnection>) {
    tokio::task::spawn(async move {
        let mut interval = interval(Duration::from_secs(3));

        loop {
            interval.tick().await;
            block_handler(cli.clone(), conn.clone()).await;
        }
    });
}

pub async fn block_handler(cli: Arc<EthCli>, conn: Arc<DatabaseConnection>) {
    let latest_block_number = cli.get_block_number().await;
    if let Some(latest_block) = Query::select_latest(&conn).await.unwrap() {
        if latest_block.number > latest_block_number as i64 {
            tracing::info!(
                "latestBlock.LatestBlockHeight: {} greater than latestBlockNumber: {}",
                latest_block.number,
                latest_block_number
            );
            return;
        }
        let current_number = latest_block.number as u64 + 1;
        let current_block = cli.get_block_with_tx(current_number).await;
        if current_block.is_none() {
            return;
        }

        let current_block = current_block.unwrap();
        tracing::info!(
            "get currentBlock blockNumber: {}, blockHash: {:#032x}, hash size: {}",
            current_block.number.unwrap(),
            current_block.hash.unwrap(),
            current_block.hash.unwrap().as_bytes().to_vec().len(),
        );

        let block_traces = cli.trace_block(current_number).await;
        let recipts = cli.get_block_receipt(current_number).await;
        let handle_models = handle_block(&current_block, &block_traces, &recipts)
            .await
            .unwrap();

        sync_to_db(&conn, handle_models).await.unwrap();
    }
}
