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

pub(crate) fn metrics(
    provider: Arc<InternalDataProvider>,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    pub async fn get_metrics(
        metric: String,
        provider: Arc<InternalDataProvider>,
        identifier: ChainId,
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
        }
    }

    let get_metrics_route = |provider: Arc<InternalDataProvider>| {
        warp::path!("metrics" / String)
            .and(warp::get())
            .and(warp::query::<ChainId>())
            .and(warp::path::end())
            .and_then(move |metric, identifier| {
                get_metrics(metric, Arc::clone(&provider), identifier)
            })
    };

    get_metrics_route(provider.clone())
}
