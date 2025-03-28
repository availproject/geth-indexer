use std::{
    collections::BTreeMap,
    time::{SystemTime, UNIX_EPOCH},
};

use alloy::rpc::types::eth::Transaction as AlloyTx;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use futures::future::join_all;
use rayon::prelude::*;
use redis::RedisResult;
use tokio::task;

use crate::{
    cache::*,
    schema::{
        chains::dsl::chains as chains_schema,
        transactions::dsl::{
            self as transactions_schema_types, transactions as transactions_schema,
        },
    },
    unix_ms_to_ist, Chain, ChainId, ConvertToHex, DatabaseConnections, Limit, Parts, Stride, Tx,
    TxAPIResponse, TxFilter, TxIdentifier, TxModel, TxResponse, TxnSummary, Type,
};

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
        tx_type: Type,
        limit: Limit,
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
        if let Some(tpe) = tx_type.tx_type.as_ref() {
            match tpe {
                Tx::Native | Tx::CrossChain => {
                    query = query.filter(transactions_schema_types::tx_type.eq(tpe.to_string()));
                }
                Tx::All => {}
            };
        }

        let result: Vec<TxModel> = query
            .limit(limit.limit.unwrap_or(10) as i64)
            .offset((identifier.page_idx.unwrap_or(0) * limit.limit.unwrap_or(10)) as i64)
            .select(TxModel::as_select())
            .load(&mut conn)
            .await
            .map_err(|_| std::io::ErrorKind::ConnectionAborted)?;

        let results: Vec<TxAPIResponse> = result
            .into_par_iter()
            .map(|tx| {
                if let Some(_) = parts.all {
                    let tx: AlloyTx = tx.into();
                    TxAPIResponse::Transaction(tx)
                } else {
                    let txn_summary = TxnSummary {
                        hash: tx.transaction_hash,
                        block_hash: tx.block_hash,
                        from: tx._from.clone(),
                        to: tx._to,
                        status: Some(1),
                        value: tx.value,
                        block_height: tx.block_number.unwrap() as u64,
                    };
                    TxAPIResponse::TxnSummary(txn_summary)
                }
            })
            .collect();

        Ok(results)
    }

    pub async fn add_txns(
        &self,
        chain_id: u64,
        tx_count: usize,
        transactions: Vec<AlloyTx>,
        tx_map: BTreeMap<String, Tx>,
    ) -> Result<(), std::io::Error> {
        let txns: Vec<TxModel> = transactions
            .iter()
            .take(10)
            .cloned()
            .collect::<Vec<_>>()
            .into_par_iter()
            .map(|transaction| {
                let tx_type = tx_map
                    .get(&transaction.hash.to_hex_string())
                    .unwrap_or(&Tx::Native);
                TxModel::from(chain_id, &transaction, tx_type)
            })
            .collect();

        {
            let mut conn = self
                .dbc
                .postgres
                .get()
                .await
                .map_err(|_| std::io::ErrorKind::ConnectionAborted)?;

            diesel::insert_into(chains_schema)
                .values(&Chain {
                    chain_id: chain_id as i64,
                    latest_tps: tx_count as i64,
                })
                .on_conflict(crate::schema::chains::chain_id)
                .do_update()
                .set(crate::schema::chains::latest_tps.eq(tx_count as i64))
                .execute(&mut conn)
                .await
                .map_err(|_| std::io::ErrorKind::ConnectionAborted)?;
        }

        let mut tasks = Vec::new();
        for chunk in txns.chunks(250) {
            let chunk = chunk.to_vec();
            let db_pool = self.dbc.postgres.clone();
            let task = task::spawn(async move {
                let mut conn = db_pool
                    .get()
                    .await
                    .map_err(|_| std::io::ErrorKind::ConnectionAborted)?;

                diesel::insert_into(transactions_schema)
                    .values(&chunk)
                    .on_conflict((
                        crate::schema::transactions::chain_id,
                        crate::schema::transactions::transaction_hash,
                    ))
                    .do_nothing()
                    .execute(&mut conn)
                    .await
                    .map_err(|_| std::io::ErrorKind::ConnectionAborted)?;

                Ok::<(), std::io::Error>(())
            });

            tasks.push(task);
        }

        let results = join_all(tasks).await;
        for res in results {
            if let Err(e) = res {
                eprintln!("Task failed: {:?}", e);
            }
        }

        Ok(())
    }

    pub async fn add_block(
        &self,
        chain_id: &u64,
        timestamp: i64,
        successful_xfers: u64,
        total_xfers: u64,
        total_native_transfers: u64,
        total_x_chain_transfers: u64,
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
                total_native_transfers,
                total_x_chain_transfers,
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

    pub async fn live_tps(
        &self,
        identifier: ChainId,
        stride: Stride,
        tx_type: Type,
    ) -> RedisResult<Vec<(u64, String)>> {
        let tps_with_timestamps = {
            let mut redis_conn = self.dbc.redis.lock().await;
            if let Some(chain_id) = identifier.chain_id {
                get_live_tps(&chain_id, stride, tx_type, &mut redis_conn)?
            } else {
                get_all_chains_live_tps_in_range(stride, tx_type, &mut redis_conn)?
            }
        };

        Ok(tps_with_timestamps)
    }

    pub async fn current_tps(&self, identifier: ChainId, tx_type: Type) -> RedisResult<u64> {
        let tps = {
            let mut redis_conn = self.dbc.redis.lock().await;
            let tps = if let Some(chain_id) = identifier.chain_id {
                get_latest_tps(&chain_id, tx_type, &mut redis_conn)?
            } else {
                get_all_chains_tps_in_range(tx_type, &mut redis_conn)?
            };
            tps as u64
        };

        Ok(tps)
    }

    pub async fn total_xfers_last_day(
        &self,
        identifier: ChainId,
        tx_type: Type,
    ) -> RedisResult<u64> {
        let tps = {
            let mut redis_conn = self.dbc.redis.lock().await;
            let tps = if let Some(chain_id) = identifier.chain_id {
                let latest_timestamp = get_latest_timestamp(&chain_id, &mut redis_conn)?;
                get_successful_xfers_in_range(
                    &chain_id,
                    86400,
                    latest_timestamp,
                    tx_type,
                    &mut redis_conn,
                )?
            } else {
                let now_duration = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .expect("SystemTime before UNIX EPOCH!");
                get_all_chains_success_xfers_in_range(
                    86400,
                    now_duration.as_secs() as i64,
                    tx_type,
                    &mut redis_conn,
                )?
            };

            tps as u64
        };

        Ok(tps)
    }

    pub async fn successful_xfers_last_day(
        &self,
        identifier: ChainId,
        tx_type: Type,
    ) -> RedisResult<u64> {
        let xfers = {
            let mut redis_conn = self.dbc.redis.lock().await;
            let xfers = if let Some(chain_id) = identifier.chain_id {
                let latest_timestamp = get_latest_timestamp(&chain_id, &mut redis_conn)?;
                get_successful_xfers_in_range(
                    &chain_id,
                    86400,
                    latest_timestamp,
                    tx_type,
                    &mut redis_conn,
                )
                .unwrap_or(0)
            } else {
                let now_duration = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .expect("SystemTime before UNIX EPOCH!");
                get_all_chains_success_xfers_in_range(
                    86400,
                    now_duration.as_secs() as i64,
                    tx_type,
                    &mut redis_conn,
                )
                .unwrap_or(0)
            };

            xfers as u64
        };

        Ok(xfers)
    }

    pub async fn transaction_volume(
        &self,
        identifier: ChainId,
        tx_type: Type,
        stride: Stride,
    ) -> RedisResult<Vec<TxResponse>> {
        let mut tx_response = Vec::new();

        {
            let mut redis_conn = self.dbc.redis.lock().await;
            let (interval, width) = if let Some(stride) = stride.stride {
                if stride == 1 {
                    // 1 hr
                    (30, 120) // interval: 30 sec (30 * 120s = 1 hr)
                } else {
                    // 10 min
                    (10, 60) // interval: 10 sec (10 * 60s = 10 min)
                }
            } else {
                (3600, 24) // interval: 15 min (24 * 1hr = 24hr)
            };

            if let Some(chain_id) = identifier.chain_id {
                let latest_timestamp = get_latest_timestamp(&chain_id, &mut redis_conn)?;

                for i in 1..width {
                    // 96 times, so iterate till last 24 hr data (96 * 15min = 24hr)
                    let success = get_successful_xfers_in_range(
                        &chain_id,
                        i * interval,
                        latest_timestamp.saturating_sub((i - 1) * interval),
                        tx_type.clone(),
                        &mut redis_conn,
                    )
                    .unwrap_or(0);

                    tx_response.push(TxResponse {
                        successful_txns: success as u64,
                        total_txns: success as u64,
                        timestamp: unix_ms_to_ist(latest_timestamp.saturating_sub(i * interval)),
                    })
                }
            } else {
                let now_duration = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .expect("SystemTime before UNIX EPOCH!");
                let latest_timestamp = now_duration.as_secs() as i64;
                for i in 1..width {
                    let success = get_all_chains_success_xfers_in_range(
                        i * interval,
                        latest_timestamp.saturating_sub((i - 1) * interval),
                        tx_type.clone(),
                        &mut redis_conn,
                    )
                    .unwrap_or(0);

                    tx_response.push(TxResponse {
                        successful_txns: success as u64,
                        total_txns: success as u64,
                        timestamp: unix_ms_to_ist(latest_timestamp.saturating_sub(i * interval)),
                    })
                }
            };
        };

        tx_response.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));

        Ok(tx_response)
    }
}
