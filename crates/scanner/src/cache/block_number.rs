use repo::dal::block::Query;
use sea_orm::DbConn;
use std::cmp::{max, min};

pub enum CacheKey {
    Min,
    Max,
}

pub struct Cache {
    min: Option<u64>,
    max: Option<u64>,
}

impl Cache {
    fn new() -> Self {
        Cache {
            min: None,
            max: None,
        }
    }

    pub fn get_max(&self) -> Option<u64> {
        self.max
    }

    pub fn get_min(&self) -> Option<u64> {
        self.min
    }

    fn handle_update(
        &mut self,
        key: CacheKey,
        old_value: Option<u64>,
        new_value: u64,
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
            _ => Err(format!("Invalid key: {}", key)),
        }
    }

    async fn handle_fallback(
        &self,
        key: CacheKey,
        conn: &DbConn,
        enabled: bool,
    ) -> Result<i64, String> {
        let result = match key {
            CacheKey::Min => Query::find_min_number(conn).await,
            CacheKey::Max => Query::find_max_number(conn).await,
            _ => return Err(format!("Invalid key: {}", key)),
        };

        if enabled {
            Ok(result)
        } else {
            Err(result)
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

        cache.handle_update(CacheKey::Min, None, 10);
        cache.handle_update(CacheKey::Max, None, 20);
        cache.handle_update(CacheKey::Min, Some(10), 5);
        cache.handle_update(CacheKey::Max, Some(20), 30);

        // let min_block_number = map_cache.handle_fallback(BlockNumberCacheKey::Min).unwrap();
        // let max_block_number = map_cache.handle_fallback(BlockNumberCacheKey::Max).unwrap();

        // println!("Min Block Number: {}", min_block_number);
        // println!("Max Block Number: {}", max_block_number);
    }
}
