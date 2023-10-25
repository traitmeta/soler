use crate::contracts::erc20::IERC20Call;
use anyhow::anyhow;
use chrono::Utc;
use entities::tokens::Model;
use repo::dal::token::{Mutation, Query};
use sea_orm::prelude::Decimal;
use sea_orm::DbConn;
use std::{collections::HashMap, sync::Arc};
use tokio::sync::Mutex;

pub struct TokenTotalSupplyOnDemand {
    ttl_in_blocks: i64,
    token_cache: Arc<Mutex<TokenCache>>,
}

impl TokenTotalSupplyOnDemand {
    pub fn new(ttl_in_blocks: i64, conn: Arc<DbConn>, erc20_call: Arc<IERC20Call>) -> Self {
        TokenTotalSupplyOnDemand {
            ttl_in_blocks,
            token_cache: Arc::new(Mutex::new(TokenCache::new(conn, erc20_call))),
        }
    }

    pub async fn trigger_fetch(
        &self,
        address: Vec<u8>,
        max_block_number: i64,
    ) -> anyhow::Result<()> {
        let mut token_cache = self.token_cache.lock().await;
        token_cache
            .fetch_and_update(address, max_block_number, self.ttl_in_blocks)
            .await?;
        Ok(())
    }
}

struct TokenCache {
    tokens: HashMap<Vec<u8>, Model>,
    conn: Arc<DbConn>,
    erc20_call: Arc<IERC20Call>,
}

impl TokenCache {
    fn new(conn: Arc<DbConn>, erc20_call: Arc<IERC20Call>) -> Self {
        TokenCache {
            tokens: HashMap::new(),
            conn,
            erc20_call,
        }
    }

    async fn fetch_and_update(
        &mut self,
        address: Vec<u8>,
        max_block_number: i64,
        ttl_in_blocks: i64,
    ) -> anyhow::Result<()> {
        let model = match self.tokens.get_key_value(&address) {
            Some((_, v)) => Some(v.clone()),
            None => match Query::find_by_hash(self.conn.as_ref(), address).await {
                Ok(v) => v,
                Err(e) => {
                    tracing::warn!(message = "find token by address", err=?e);
                    None
                }
            },
        };

        if let Some(mut token) = model {
            self.update_total_supply(&mut token, max_block_number, ttl_in_blocks)
                .await
                .unwrap();
            return Ok(());
        }

        Ok(())
    }

    async fn update_total_supply(
        &self,
        token: &mut Model,
        max_block_number: i64,
        ttl_in_blocks: i64,
    ) -> Result<(), anyhow::Error> {
        if token.total_supply_updated_at_block.is_none()
            || max_block_number - token.total_supply_updated_at_block.unwrap() > ttl_in_blocks
        {
            let token_address_hash =
                format!("0x{}", hex::encode(token.contract_address_hash.clone()));
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
            let updated_token = Mutation::update_total_supply(self.conn.as_ref(), token).await;
            match updated_token {
                Ok(_) => Ok(()),
                Err(e) => {
                    tracing::warn!(message = "update total supply", err=?e);
                    Err(anyhow!("update total supply failed"))
                }
            }
        } else {
            Ok(())
        }
    }
}
