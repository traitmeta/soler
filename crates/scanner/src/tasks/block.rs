use std::time::Duration;

use repo::dal::block::Query;
use sea_orm::DbConn;
use tokio::time::interval;

use crate::evms::eth::EthCli;
use crate::handler::block;

pub async fn sync_task(cli: EthCli, conn: &DbConn) {
    let mut interval = interval(Duration::from_secs(3));
    loop {
        interval.tick().await;

        let latest_block_number = cli.get_block_number().await;
        if let Some(latest_block) = Query::select_latest(&conn).await.unwrap() {
            if latest_block.number > latest_block_number as i64 {
                tracing::info!(
                    "latestBlock.LatestBlockHeight: {} greater than latestBlockNumber: {}",
                    latest_block.number,
                    latest_block_number
                );
                continue;
            }
            let current_block = cli.get_block_with_tx(latest_block.number as u64 + 1).await;

            tracing::info!(
                "get currentBlock blockNumber: {}, blockHash: {:#032x}, hash size: {}",
                current_block.number.unwrap(),
                current_block.hash.unwrap(),
                current_block.hash.unwrap().as_bytes().to_vec().len(),
            );

            let block_traces = cli.trace_block(latest_block.number as u64 + 1).await;

            block::handle_block(&current_block, &block_traces)
                .await
                .unwrap();
        }
    }
}
