use redis::RedisResult;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::unix_ms_to_ist;
use crate::TxResponse;
use crate::{cache::*, ChainId, DatabaseConnections};

#[derive(Clone)]
pub struct InternalDataProvider {
    pub dbc: DatabaseConnections,
}

impl InternalDataProvider {
    pub async fn new() -> Result<Self, std::io::Error> {
        Ok(InternalDataProvider {
            dbc: DatabaseConnections::init().await?,
        })
    }

    pub async fn add_block(
        &self,
        chain_id: &u64,
        timestamp: i64,
        successful_xfers: u64,
        total_xfers: u64,
        tx_count: usize,
        height: u64,
    ) -> RedisResult<()> {
        {
            let mut redis_conn = self.dbc.redis.lock().await;
            add_block(
                chain_id,
                timestamp,
                successful_xfers,
                total_xfers,
                tx_count as u64,
                height,
                &mut redis_conn,
            )
        }
    }

    pub async fn get_latest_height(&self, id: &u64) -> RedisResult<u64> {
        let height = {
            let mut redis_conn = self.dbc.redis.lock().await;
            match get_latest_height(id, &mut redis_conn) {
                Ok(ht) => Ok(ht),
                Err(_) => Ok(0),
            }
        };

        height
    }

    pub async fn current_tps(&self, identifier: ChainId) -> RedisResult<u64> {
        let tps = {
            let mut redis_conn = self.dbc.redis.lock().await;
            let tps = if let Some(chain_id) = identifier.chain_id {
                get_latest_tps(&chain_id, &mut redis_conn)?
            } else {
                get_all_chains_tps_in_range(&mut redis_conn)?
            };
            tps as u64
        };

        Ok(tps)
    }

    pub async fn total_xfers_last_day(&self, identifier: ChainId) -> RedisResult<u64> {
        let tps = {
            let mut redis_conn = self.dbc.redis.lock().await;
            let tps = if let Some(chain_id) = identifier.chain_id {
                get_successful_xfers_in_range(&chain_id, 86400, &mut redis_conn)?
            } else {
                get_all_chains_success_xfers_in_range(86400, &mut redis_conn)?
            };

            tps as u64
        };

        Ok(tps)
    }

    pub async fn successful_xfers_last_day(&self, identifier: ChainId) -> RedisResult<u64> {
        let xfers = {
            let mut redis_conn = self.dbc.redis.lock().await;
            let xfers = if let Some(chain_id) = identifier.chain_id {
                get_successful_xfers_in_range(&chain_id, 86400, &mut redis_conn).unwrap_or(0)
            } else {
                get_all_chains_success_xfers_in_range(86400, &mut redis_conn).unwrap_or(0)
            };

            xfers as u64
        };

        Ok(xfers)
    }

    pub async fn transaction_volume(&self, identifier: ChainId) -> RedisResult<Vec<TxResponse>> {
        let mut tx_response = Vec::new();

        {
            let now_duration = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("SystemTime before UNIX EPOCH!");

            let now_seconds = now_duration.as_secs() as i64;
            let mut redis_conn = self.dbc.redis.lock().await;
            if let Some(chain_id) = identifier.chain_id {
                for i in 1..20 {
                    let success =
                        get_successful_xfers_in_range(&chain_id, i, &mut redis_conn).unwrap_or(0);

                    tx_response.push(TxResponse {
                        successful_txns: success as u64,
                        total_txns: success as u64,
                        timestamp: unix_ms_to_ist(now_seconds.saturating_sub(86400)),
                    })
                }
            } else {
                for i in 1..20 {
                    let success =
                        get_all_chains_success_xfers_in_range(i, &mut redis_conn).unwrap_or(0);

                    tx_response.push(TxResponse {
                        successful_txns: success as u64,
                        total_txns: success as u64,
                        timestamp: unix_ms_to_ist(now_seconds.saturating_sub(i)),
                    })
                }
            };
        };

        Ok(tx_response)
    }
}
