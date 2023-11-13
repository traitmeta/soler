use anyhow::anyhow;
use repo::dal::block::Query;
use sea_orm::DbConn;
use std::cmp::{max, min};

#[derive(Debug)]
pub enum CacheKey {
    Min,
    Max,
}

pub struct Cache {
    min: Option<i64>,
    max: Option<i64>,
}

impl Default for Cache {
    fn default() -> Self {
        Self::new()
    }
}

impl Cache {
    pub fn new() -> Self {
        Cache {
            min: None,
            max: None,
        }
    }

    pub fn get_max(&self) -> Option<i64> {
        self.max
    }

    pub fn get_min(&self) -> Option<i64> {
        self.min
    }

    pub fn handle_update(
        &mut self,
        key: CacheKey,
        old_value: Option<i64>,
        new_value: i64,
    ) -> Result<(), String> {
        match key {
            CacheKey::Min => {
                self.min = Some(match old_value {
                    Some(old) => min(new_value, old),
                    None => new_value,
                });
                Ok(())
            }
            CacheKey::Max => {
                self.max = Some(match old_value {
                    Some(old) => max(new_value, old),
                    None => new_value,
                });
                Ok(())
            }
        }
    }

    pub async fn handle_fallback(
        &self,
        key: CacheKey,
        conn: &DbConn,
        enabled: bool,
    ) -> anyhow::Result<i64> {
        let result = match key {
            CacheKey::Min => Query::find_min_number(conn).await,
            CacheKey::Max => Query::find_max_number(conn).await,
        };

        if enabled {
            Ok(result?)
        } else {
            Err(anyhow!("unenbaled"))
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::cache::block_number::{Cache, CacheKey};
    // use std::time::Duration;

    #[test]
    fn test_cache() {
        let mut cache = Cache::new();
        // let ttl_check_interval = 10;
        // let global_ttl = 60;

        // let map_cache = MapCache::new(cache)
        //     .name("block_number")
        //     .keys(vec![BlockNumberCacheKey::Min, BlockNumberCacheKey::Max])
        //     .ttl_check_interval(Duration::from_secs(ttl_check_interval))
        //     .global_ttl(Duration::from_secs(global_ttl))
        //     .build();

        let _ = cache.handle_update(CacheKey::Min, None, 10);
        let _ = cache.handle_update(CacheKey::Max, None, 20);
        let _ = cache.handle_update(CacheKey::Min, Some(10), 5);
        let _ = cache.handle_update(CacheKey::Max, Some(20), 30);

        // let min_block_number = map_cache.handle_fallback(BlockNumberCacheKey::Min).unwrap();
        // let max_block_number = map_cache.handle_fallback(BlockNumberCacheKey::Max).unwrap();

        // println!("Min Block Number: {}", min_block_number);
        // println!("Max Block Number: {}", max_block_number);
    }
}
