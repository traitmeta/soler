use std::time::Duration;

use anyhow::bail;
use entities::{
    addresses::Model as AddressModel, blocks::Model as BlockModel,
    internal_transactions::Model as InnerTransactionModel, logs::Model as LogModel,
    token_transfers::Model as TokenTransferModel, tokens::Model as TokenModel,
    transactions::Model as TransactionModel, withdrawals::Model as WithdrawModel,
};
use ethers::types::{Block, Trace, Transaction, TransactionReceipt, TxHash, H256, U64};
use repo::dal::block::Query;
use repo::dal::{
    address::Mutation as AddressMutation,
    block::{Mutation as BlockMutation, Query as BlockQuery},
    event::Mutation as EventMutation,
    internal_transaction::Mutation as InnerTransactionMutation,
    token::Mutation as TokenMutation,
    token_transfer::Mutation as TokenTransferMutation,
    transaction::Mutation as TransactionMutation,
    withdrawal::Mutation as WithdrawalMutation,
};
use sea_orm::DbConn;
use tokio::time::interval;

use crate::evms::eth::EthCli;
use crate::handler::block::{self, handle_block};

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
            let current_number = latest_block.number as u64 + 1;
            let current_block = cli.get_block_with_tx(current_number).await;

            tracing::info!(
                "get currentBlock blockNumber: {}, blockHash: {:#032x}, hash size: {}",
                current_block.number.unwrap(),
                current_block.hash.unwrap(),
                current_block.hash.unwrap().as_bytes().to_vec().len(),
            );

            let block_traces = cli.trace_block(current_number).await;
            let recipts = cli.get_block_receipt(current_number).await;

            let (block_model, handle_models) =
                handle_block(&current_block, &block_traces, &recipts)
                    .await
                    .unwrap();

            sync_to_db(&block_model, handle_models).await?;
        }
    }
}

