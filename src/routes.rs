use std::sync::Arc;
use axum::{extract::{ Path, State}, response as response_axum, Json};

use crate::error::IndexerError;



pub async fn health_check() -> Result<impl response_axum::IntoResponse, IndexerError> {
    Ok("Live".to_string())
}

pub async fn get_tps(Path(chain_id): Path<u64>, State(db): State<Arc<sled::Db>>) -> Result<impl response_axum::IntoResponse, IndexerError> {
    match db.get(chain_id.to_be_bytes()) {
        Ok(Some(res)) => {
            let latest_block_number = u64::from_be_bytes(res.as_ref().try_into().unwrap());
            if let Ok(tx) = db.get(format!("{}:{}", chain_id, latest_block_number)) {
                let running_tx_count: usize = match tx {
                    Some(ivec) if ivec.len() == 8 => {
                        let bytes: [u8; 8] = ivec.as_ref().try_into().unwrap(); 
                        usize::from_be_bytes(bytes)
                    }
                    _ => {
                        return Ok(Json(serde_json::json!({ "error": "Database error" })));
                    }
                };
                let tps = running_tx_count.saturating_div(latest_block_number as usize);
                Ok(Json(serde_json::json!({ "chain_id": chain_id, "tps": tps })))
            } else {
                Ok(Json(serde_json::json!({ "error": "Chain ID not found" })))
            }
        },
        Ok(None) => Ok(Json(serde_json::json!({ "error": "Chain ID not found" }))),
        Err(_) => Ok(Json(serde_json::json!({ "error": "Database error" }))),
    }
}
