use db::{provider::InternalDataProvider, types::*};
use std::{convert::Infallible, str::FromStr, sync::Arc};
use warp::{self, http, Filter};

use crate::error::IndexerError;

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
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    pub async fn get_transactions(
        limit: Limit,
        parts: Parts,
        tx_identifier: TxIdentifier,
        tx_filter: TxFilter,
        internal_provider: Arc<InternalDataProvider>,
    ) -> Result<impl warp::Reply, warp::Rejection> {
        let window = limit.limit.unwrap_or(MAX_WINDOW_SIZE);
        if window == 0 || window > MAX_WINDOW_SIZE {
            return Err(warp::reject::custom(
                IndexerError::ProviderError("failed to deserialize".to_string()),
            ));
        }

        if (tx_identifier.field_count() > 1) || (parts.field_count() > 1)
        {
            return Err(warp::reject::custom(
                IndexerError::ProviderError("failed to deserialize".to_string()),
            ));
        }

        let tx_responses = match internal_provider
            .get_txs(
                tx_identifier,
                tx_filter,
                parts,
            )
            .await
        {
            Ok(transactions) => transactions,
            Err(_) => {
                return Err(warp::reject::custom(
                    IndexerError::ProviderError("failed to get response from db".to_string()),
                ));
            }
        };

        Ok(warp::reply::json(&tx_responses))
    }

    let transactions_route = |internal_provider: Arc<InternalDataProvider>| {
        warp::get()
            .and(warp::path("transactions"))
            .and(warp::query::<TxIdentifier>())
            .and(warp::query::<TxFilter>())
            .and(warp::query::<Parts>())
            .and(warp::query::<Limit>())
            .and(warp::path::end())
            .and_then(move |tx_identifier, tx_filter, parts, limit| {
                get_transactions(limit, parts, tx_identifier, tx_filter, internal_provider.clone())
            })
    };

    transactions_route(internal_provider.clone())
}

pub(crate) fn metrics(
    provider: Arc<InternalDataProvider>,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    pub async fn get_metrics(
        metric: String,
        provider: Arc<InternalDataProvider>,
        identifier: ChainId,
        stride: Stride,
    ) -> Result<impl warp::Reply, warp::Rejection> {
        let performance_metric: Metric = match Metric::from_str(&metric) {
            Ok(f) => f,
            Err(e) => {
                return Err(warp::reject::custom(IndexerError::DeserializationError(e)));
            }
        };

        match performance_metric {
            Metric::CurrentTPS => {
                let tps = provider
                    .current_tps(identifier)
                    .await
                    .map_err(|e| IndexerError::RedisError(e))?;

                Ok(warp::reply::json(&tps))
            }
            Metric::TransactionVolume => {
                let tx_volume = provider
                    .transaction_volume(identifier)
                    .await
                    .map_err(|e| IndexerError::RedisError(e))?;
                Ok(warp::reply::json(&tx_volume))
            }
            Metric::TotalTransactions => {
                let total_txns = provider
                    .total_xfers_last_day(identifier)
                    .await
                    .map_err(|e| IndexerError::RedisError(e))?;
                Ok(warp::reply::json(&total_txns))
            }
            Metric::SuccessfulTransfers => {
                let successful_xfers = provider
                    .successful_xfers_last_day(identifier)
                    .await
                    .map_err(|e| IndexerError::RedisError(e))?;
                Ok(warp::reply::json(&successful_xfers))
            }
            Metric::LiveTPS => {
                let tps = provider
                    .live_tps(identifier, stride)
                    .await
                    .map_err(|e| IndexerError::RedisError(e))?;

                Ok(warp::reply::json(&tps))
            }
        }
    }

    let get_metrics_route = |provider: Arc<InternalDataProvider>| {
        warp::path!("metrics" / String)
            .and(warp::get())
            .and(warp::query::<ChainId>())
            .and(warp::query::<Stride>())
            .and(warp::path::end())
            .and_then(move |metric, identifier, stride| {
                get_metrics(metric, Arc::clone(&provider), identifier, stride)
            })
    };

    get_metrics_route(provider.clone())
}
