use std::convert::Infallible;

use db::RedisError;
use reqwest::Error as ReqwestError;
use serde::{Deserialize, Serialize};
use serde_json::Error as SerdeError;
use thiserror::Error;
use tokio::task::JoinError;
use warp::{self, http, hyper::StatusCode};

#[derive(Debug, Serialize, Deserialize)]
pub struct ResponseError {
    pub code: u16,
    pub message: String,
}

#[derive(Debug, Error)]
pub enum IndexerError {
    #[error("Deserialization Error")]
    DeserializationError(#[from] SerdeError),

    #[error("Reqwest Error: {0}")]
    ReqwestError(#[from] ReqwestError),

    #[error("Tokio Join Error: {0}")]
    TokioJoinError(#[from] JoinError),

    #[error("Redis Error: {0}")]
    RedisError(#[from] RedisError),
    
    #[error("External Provider Error")]
    ProviderError(String),
}

impl warp::reject::Reject for IndexerError {}

pub(crate) async fn handle_rejection(
    err: warp::reject::Rejection,
) -> Result<impl warp::Reply, Infallible> {
    let (code, message): (StatusCode, &str) = match err.find() {
        Some(IndexerError::DeserializationError(_)) => {
            (StatusCode::BAD_REQUEST, "Deserialization Error")
        }
        Some(IndexerError::ReqwestError(_)) => (StatusCode::BAD_REQUEST, "Reqwest Error"),
        Some(IndexerError::TokioJoinError(_)) => (StatusCode::BAD_REQUEST, "Tokio Join Error"),
        Some(IndexerError::RedisError(_)) => (StatusCode::BAD_REQUEST, "Redis Error"),
        Some(IndexerError::ProviderError(_)) => (StatusCode::BAD_REQUEST, "External Provider Error"),
        None => (StatusCode::BAD_REQUEST, "Unknown Error Code"),
    };

    let error = serde_json::to_string(&ResponseError {
        code: code.as_u16(),
        message: message.to_string(),
    })
    .unwrap();

    Ok(http::Response::builder().status(code).body(error))
}
