use alloy::rpc::types::eth::Transaction;
use diesel::{ExpressionMethods, QueryDsl, SelectableHelper};
use diesel_async::RunQueryDsl;
use rayon::prelude::*;
use redis::RedisResult;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::schema::transactions::dsl::{
    self as transactions_schema_types, transactions as transactions_schema,
};
use crate::schema::chains::dsl::chains as chains_schema;
use crate::{Chain, TxnSummary};
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

    // pub async fn get_txs(
    //     &mut self,
    //     chain_id: &u64,
    //     filter: TxIdentifier,
    // ) -> Result<Vec<TxAPIResponse>, std::io::Error> {
    //     let mut conn = self
    //             .dbc
    //             .postgres
    //             .get()
    //             .await
    //             .map_err(|_| std::io::ErrorKind::ConnectionAborted)?;
        
    //     let mut query = transactions_schema.into_boxed();        
    //     query = query.order(transactions_schema_types::block_number.desc());
       
    //     if let Some(tx_hash) = filter.tx_hash.as_ref() {
    //         query = query.filter(transactions_schema_types::transaction_hash.eq(tx_hash));
    //     }
        
    //     let result: Vec<TransactionModel> = query
    //         .limit(25_i64)
    //         .offset(
    //             (filter.page_idx.unwrap_or_default() * 10)
    //                 as i64,
    //         )
    //         .select((
    //             TransactionModel::as_select(),
    //         ))
    //         .load(&mut conn)
    //         .await
    //         .map_err(|_| std::io::ErrorKind::ConnectionAborted)?;

    //     let mut summaries = Vec::new();
    //     for (tx, receipt) in result {
    //         let (node_name, block_number, timestamp) =
    //             get_additional_tx_info(&mut self.postgres, &tx).await?;
    //         let transaction: AlloyTransaction = tx.into();
    //         let txn_summary = TxnSummary {
    //             timestamp,
    //             hash: transaction.hash,
    //             signer: transaction,
    //             block_number,
    //             from: Some(Address(*transaction.from)),
    //             to: transaction.to,
    //             receipt: ReceiptSummary {
    //                 gas_used: Some(receipt.gas_used.parse()?),
    //                 status: receipt.transaction_status.parse()?,
    //                 logs: log_map.remove(&transaction.hash).unwrap_or_default(),
    //             },
    //             max_fee_per_gas: transaction.max_fee_per_gas.map(|fee| U256::from(fee)),
    //         };

    //         summaries.push(txn_summary);
    //     }

    //     Ok(summaries)
    // }

    // pub async fn get_txns(
    //     &self,
    //     chain_id: u64,
    //     tx_identifier: TxIdentifier,
    // ) -> Result<(), std::io::Error> {
    //     let tx_response = {
    //         let mut conn = self
    //             .dbc
    //             .postgres
    //             .get()
    //             .await
    //             .map_err(|_| std::io::ErrorKind::ConnectionAborted)?;

    //         let result = if tx_identifier.tx_hash.is_some() {
    //             transactions_schema
    //                 .filter(transactions_schema_types::transaction_hash.eq(tx_hash.to_hex_string()))
    //                 .filter(transactions_schema_types::node_id.eq_any(&self.node_ids))
    //                 .select(TransactionModel::as_select())
    //                 .load(self.postgres.get_conn().await?.as_mut())
    //                 .await?
    //         }

    //         let result = transactions_schema
    //             .filter(transactions_schema_types::transaction_hash.eq(tx_hash.to_hex_string()))
    //             .filter(transactions_schema_types::node_id.eq_any(&self.node_ids))
    //             .select(TransactionModel::as_select())
    //             .load(self.postgres.get_conn().await?.as_mut())
    //             .await?;

    

    //         diesel::insert_into(transactions_schema)
    //             .values(&txns)
    //             .execute(&mut conn)
    //             .await
    //             .map_err(|_| std::io::ErrorKind::ConnectionAborted)?;
    //     }

    //     Ok(())
    // }



    // pub async fn add_txns(
    //     &self,
    //     chain_id: u64,
    //     tx_count: usize,
    //     transactions: Vec<Transaction>,
    // ) -> Result<(), std::io::Error> {
    //     let txns: Vec<TransactionModel> = transactions
    //         .par_iter()
    //         .map(|transaction| TransactionModel::from(chain_id, transaction))
    //         .collect();

    //     {
    //         let mut conn = self
    //             .dbc
    //             .postgres
    //             .get()
    //             .await
    //             .map_err(|_| std::io::ErrorKind::ConnectionAborted)?;

    //         diesel::insert_into(chains_schema)
    //             .values(&Chain { chain_id: chain_id as i64, latest_tps: tx_count as i64 })
    //             .execute(&mut conn)
    //             .await
    //             .map_err(|_| std::io::ErrorKind::ConnectionAborted)?;

    //         diesel::insert_into(transactions_schema)
    //             .values(&txns)
    //             .execute(&mut conn)
    //             .await
    //             .map_err(|_| std::io::ErrorKind::ConnectionAborted)?;
    //     }

    //     Ok(())
    // }

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
