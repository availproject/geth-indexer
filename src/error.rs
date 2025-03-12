use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use reqwest::Error as ReqwestError;
use serde::{Serialize, Deserialize};
use serde_json::{json, Error as SerdeError, Value};
use thiserror::Error;
use tokio::task::JoinError;

#[derive(Debug, Serialize, Deserialize)]
pub struct ResponseError {
    pub code: i64,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<String>,
}

#[derive(Debug, Error)]
pub enum IndexerError {
    #[error("Deserialization Error")]
    DeserializationError(#[from] SerdeError),

    #[error("Reqwest Error: {0}")]
    ReqwestError(#[from] ReqwestError),

    #[error("Tokio Join Error: {0}")]
    TokioJoinError(#[from] JoinError),
}

impl IndexerError {
    fn into_status_and_value(self) -> (StatusCode, Value) {
        let status = match self {
            IndexerError::DeserializationError(_) => StatusCode::BAD_REQUEST,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        };
        let message: String = match status {
            StatusCode::INTERNAL_SERVER_ERROR => "Internal server error".into(),
            _ => format!("{}", self),
        };
        let error = ResponseError {
            code: self.get_error_code(),
            message,
            data: None,
        };
        (status, json!({"error": error, "id": ()}))
    }

    pub fn get_error_code(&self) -> i64 {
        match self {
            Self::DeserializationError(_) => DESERIALIZATION_ERROR,
            Self::ReqwestError(_) => REQWEST_ERROR,
            Self::TokioJoinError(_) => JOIN_ERROR,
        }
    }
}

const DESERIALIZATION_ERROR: i64 = 1001;
const REQWEST_ERROR: i64 = 1002;
const JOIN_ERROR: i64 = 1004;

impl IntoResponse for IndexerError {
    fn into_response(self) -> Response {
        let (status, value) = self.into_status_and_value();
        (status, Json(value)).into_response()
    }
}
