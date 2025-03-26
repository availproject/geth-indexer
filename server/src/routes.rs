use alloy::{primitives::TxHash, providers::Provider};
use db::{provider::InternalDataProvider, types::*};
use std::{collections::BTreeMap, convert::Infallible, str::FromStr, sync::Arc};
use warp::{self, http, Filter};

use crate::{error::IndexerError, indexer::ExternalProvider};

pub(crate) fn index_route(
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    async fn index_page_handler() -> Result<impl warp::Reply, Infallible> {
        let body = "Geth Indexer.".to_string();
        Ok(http::Response::builder().body(body))
    }

    warp::path!().and(warp::get()).and_then(index_page_handler)
}

pub(crate) fn transactions(
    internal_provider: Arc<InternalDataProvider>,
    external_provider_map: BTreeMap<u64, ExternalProvider>,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    pub async fn get_transactions(
        limit: Limit,
        tx_type: Type,
        parts: Parts,
        tx_identifier: TxIdentifier,
        tx_filter: TxFilter,
        internal_provider: Arc<InternalDataProvider>,
        external_provider_map: BTreeMap<u64, ExternalProvider>,
    ) -> Result<impl warp::Reply, warp::Rejection> {
        let window = limit.limit.unwrap_or(MAX_WINDOW_SIZE);
        if window == 0 || window > MAX_WINDOW_SIZE {
            return Err(warp::reject::custom(IndexerError::ProviderError(
                "failed to deserialize".to_string(),
            )));
        }

        if (tx_identifier.field_count() > 1) || (parts.field_count() > 1) {
            return Err(warp::reject::custom(IndexerError::ProviderError(
                "failed to deserialize".to_string(),
            )));
        }

        if let Some(tx_hash_str) = tx_identifier.tx_hash.clone() {
            let hash = TxHash::from_str(&tx_hash_str).map_err(|_| {
                warp::reject::custom(IndexerError::ProviderError(format!(
                    "Invalid transaction hash: {}",
                    tx_hash_str
                )))
            })?;

            if let Some(chain_id) = tx_filter.chain_id {
                let provider = external_provider_map.get(&chain_id).ok_or_else(|| {
                    warp::reject::custom(IndexerError::ProviderError(format!(
                        "No provider found for chain ID: {}",
                        chain_id
                    )))
                })?;

                let tx = provider
                    .get_transaction_by_hash(hash)
                    .await
                    .map_err(|_| {
                        warp::reject::custom(IndexerError::ProviderError(format!(
                            "Failed to fetch transaction from provider for chain {}",
                            chain_id
                        )))
                    })?
                    .ok_or_else(|| {
                        warp::reject::custom(IndexerError::ProviderError(format!(
                            "Transaction not found on chain {}",
                            chain_id
                        )))
                    })?;

                return Ok(warp::reply::json(&vec![TxAPIResponse::Transaction(tx)]));
            }

            for (_chain_id, provider) in external_provider_map {
                match provider.get_transaction_by_hash(hash).await {
                    Ok(Some(tx)) => {
                        return Ok(warp::reply::json(&vec![TxAPIResponse::Transaction(tx)]));
                    }
                    Ok(None) => continue,
                    Err(_e) => {
                        continue;
                    }
                }
            }
        }

        let tx_responses = match internal_provider
            .get_txs(tx_identifier, tx_filter, parts, tx_type, limit)
            .await
        {
            Ok(transactions) => transactions,
            Err(_) => {
                return Err(warp::reject::custom(IndexerError::ProviderError(
                    "failed to get response from db".to_string(),
                )));
            }
        };

        Ok(warp::reply::json(&tx_responses))
    }

    let transactions_route =
        |internal_provider: Arc<InternalDataProvider>,
         external_provider_map: BTreeMap<u64, ExternalProvider>| {
            warp::get()
                .and(warp::path("transactions"))
                .and(warp::query::<TxIdentifier>())
                .and(warp::query::<TxFilter>())
                .and(warp::query::<Parts>())
                .and(warp::query::<Type>())
                .and(warp::query::<Limit>())
                .and(warp::path::end())
                .and_then(move |tx_identifier, tx_filter, parts, tx_type, limit| {
                    get_transactions(
                        limit,
                        tx_type,
                        parts,
                        tx_identifier,
                        tx_filter,
                        internal_provider.clone(),
                        external_provider_map.clone(),
                    )
                })
        };

    transactions_route(internal_provider.clone(), external_provider_map.clone())
}

pub(crate) fn metrics(
    provider: Arc<InternalDataProvider>,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    pub async fn get_metrics(
        metric: String,
        provider: Arc<InternalDataProvider>,
        identifier: ChainId,
        stride: Stride,
        tx_type: Type,
    ) -> Result<impl warp::Reply, warp::Rejection> {
        let performance_metric: Metric = match Metric::from_str(&metric) {
            Ok(f) => f,
            Err(e) => {
                return Err(warp::reject::custom(IndexerError::DeserializationError(e)));
            }
        };

        match performance_metric {
            Metric::CurrentTPS => {
                let tps = provider.current_tps(identifier, tx_type).await.unwrap_or(0);

                Ok(warp::reply::json(&tps))
            }
            Metric::TransactionVolume => {
                let tx_volume = provider
                    .transaction_volume(identifier, tx_type, stride)
                    .await
                    .unwrap_or(Vec::new());
                Ok(warp::reply::json(&tx_volume))
            }
            Metric::TotalTransactions => {
                let total_txns = provider
                    .total_xfers_last_day(identifier, tx_type)
                    .await
                    .unwrap_or(0);
                Ok(warp::reply::json(&total_txns))
            }
            Metric::SuccessfulTransfers => {
                let successful_xfers = provider
                    .successful_xfers_last_day(identifier, tx_type)
                    .await
                    .unwrap_or(0);
                Ok(warp::reply::json(&successful_xfers))
            }
            Metric::LiveTPS => {
                let tps = provider
                    .live_tps(identifier, stride, tx_type)
                    .await
                    .unwrap_or(Vec::new());
                Ok(warp::reply::json(&tps))
            }
        }
    }

    let get_metrics_route = |provider: Arc<InternalDataProvider>| {
        warp::path!("metrics" / String)
            .and(warp::get())
            .and(warp::query::<ChainId>())
            .and(warp::query::<Stride>())
            .and(warp::query::<Type>())
            .and(warp::path::end())
            .and_then(move |metric, identifier, stride, tx_type| {
                get_metrics(metric, Arc::clone(&provider), identifier, stride, tx_type)
            })
    };

    get_metrics_route(provider.clone())
}
