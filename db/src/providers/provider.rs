use alloy::rpc::types::eth::Transaction as AlloyTx;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use rayon::prelude::*;
use redis::RedisResult;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::schema::transactions::dsl::{
    self as transactions_schema_types, transactions as transactions_schema,
};
use crate::{Parts, TxFilter, TxnSummary};
use crate::Stride;
use crate::TxAPIResponse;
use crate::TxIdentifier;
use crate::TxResponse;
use crate::{cache::*, ChainId, DatabaseConnections};
use crate::{unix_ms_to_ist, TransactionModel};

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

    pub async fn get_txs(
        &self,
        identifier: TxIdentifier,
        filter: TxFilter,
        parts: Parts,
    ) -> Result<Vec<TxAPIResponse>, std::io::Error> {
        let mut conn = self
                .dbc
                .postgres
                .get()
                .await
                .map_err(|_| std::io::ErrorKind::ConnectionAborted)?;
        
        let mut query = transactions_schema.into_boxed();        
        query = query.order(transactions_schema_types::block_number.desc());
        if let Some(chain_id) = filter.chain_id.as_ref() {
            query = query.filter(transactions_schema_types::chain_id.eq(chain_id.clone() as i64));
        }
        if let Some(tx_hash) = identifier.tx_hash.as_ref() {
            query = query.filter(transactions_schema_types::transaction_hash.eq(tx_hash));
        }

        let result: Vec<TransactionModel> = query
            .limit(25_i64)
            .offset(
                (identifier.page_idx.unwrap_or_default() * 10)
                    as i64,
            )
            .select(TransactionModel::as_select())
            .load(&mut conn)
            .await
            .map_err(|_| std::io::ErrorKind::ConnectionAborted)?;

        let mut results = Vec::new();
        for tx in result {
            if let Some(_) = parts.all {
                let tx: AlloyTx = tx.into();
                results.push(TxAPIResponse::Transaction(tx));
            } else {
                let txn_summary = TxnSummary {
                    hash: tx.transaction_hash,
                    signer: tx._from,
                    status: Some(1),
                    value: tx.value,
                    block_height: tx.block_number.unwrap() as u64,
                };
                results.push(TxAPIResponse::TxnSummary(txn_summary));
            };
        }

        Ok(results)
    }

    pub async fn add_txns(
        &self,
        chain_id: u64,
        tx_count: usize,
        transactions: Vec<AlloyTx>,
    ) -> Result<(), std::io::Error> {
        let txns: Vec<TransactionModel> = transactions
            .par_iter()
            .map(|transaction| TransactionModel::from(chain_id, transaction))
            .collect();

        {
            let mut conn = self
                .dbc
                .postgres
                .get()
                .await
                .map_err(|_| std::io::ErrorKind::ConnectionAborted)?;

            diesel::insert_into(transactions_schema)
                .values(&txns)
                .execute(&mut conn)
                .await
                .map_err(|_| std::io::ErrorKind::ConnectionAborted)?;
        }

        Ok(())
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

    pub async fn live_tps(&self, identifier: ChainId, stride: Stride) -> RedisResult<Vec<u64>> {
        let tps = {
            let mut redis_conn = self.dbc.redis.lock().await;
            if let Some(chain_id) = identifier.chain_id {
                get_live_tps(&chain_id, stride, &mut redis_conn)?
            } else {
                get_all_chains_live_tps_in_range(stride, &mut redis_conn)?
            }
        };

        Ok(tps)
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
