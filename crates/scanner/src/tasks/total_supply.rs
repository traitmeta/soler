use super::publisher::{BroadcastType, Publisher};
use crate::{cache::block_number::Cache, common::err::FetchError, contracts::erc20::IERC20Call};
use chrono::Utc;
use common::chain_ident;
use repo::dal::token::{Mutation, Query};
use sea_orm::prelude::Decimal;
use sea_orm::DbConn;
use std::sync::Arc;

pub struct TokenTotalSupplyOnDemand {
    ttl_in_blocks: i64,
    erc20_call: Arc<IERC20Call>,
    chain_publisher: Arc<Publisher>,
    conn: Arc<DbConn>,
    block_number_cache: Arc<Cache>,
}

impl TokenTotalSupplyOnDemand {
    pub fn new(
        ttl_in_blocks: i64,
        erc20_call: Arc<IERC20Call>,
        chain_publisher: Arc<Publisher>,
        conn: Arc<DbConn>,
        block_number_cache: Arc<Cache>,
    ) -> Self {
        TokenTotalSupplyOnDemand {
            ttl_in_blocks,
            erc20_call,
            chain_publisher,
            conn,
            block_number_cache,
        }
    }

    pub async fn trigger_fetch(&self, address: Vec<u8>) -> Result<(), FetchError> {
        self.fetch_and_update(address).await?;
        Ok(())
    }

    async fn fetch_and_update(&self, address: Vec<u8>) -> Result<(), FetchError> {
        let token = Query::find_by_hash(self.conn.as_ref(), address)
            .await
            .map_err(|_| FetchError::TokenNotFound)?;

        let max_block_number = self.block_number_cache.get_max().unwrap();
        let mut token = token.unwrap();
        if token.total_supply_updated_at_block.is_none()
            || max_block_number - token.total_supply_updated_at_block.unwrap() > self.ttl_in_blocks
        {
            let token_address_hash = chain_ident!(token.contract_address_hash.clone());

            let total_supply = self
                .erc20_call
                .total_supply(token_address_hash.as_str())
                .await
                .unwrap();
            token.total_supply_updated_at_block = Some(max_block_number);
            token.updated_at = Utc::now().naive_utc();
            token.total_supply = Some(Decimal::from_i128_with_scale(
                total_supply.as_u128() as i128,
                0,
            ));

            let updated_token = Mutation::update_total_supply(self.conn.as_ref(), &token)
                .await
                .map_err(|e| FetchError::TokenUpdateSupply {
                    src: total_supply.to_string(),
                    err: e,
                })?;

            self.chain_publisher
                .broadcast(
                    vec![(
                        "total_supply_update".to_string(),
                        serde_json::to_string(&updated_token).unwrap(),
                    )],
                    BroadcastType::OnDamend,
                )
                .await;

            Ok(())
        } else {
            Ok(())
        }
    }
}
