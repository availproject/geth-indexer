use alloy::eips::BlockNumberOrTag;
use alloy::providers::Provider;
use async_std::task::sleep;
use std::{sync::Arc, time};

use crate::error::IndexerError;
use crate::indexer::NodeProvider;

pub(crate) async fn catch_up_blocks(db: Arc<sled::Db>, provider: NodeProvider, chain_id: &u64) -> Result<(), IndexerError> {
    let (mut indexer_block_height, mut query_param) = db
        .get(chain_id.to_be_bytes())
        .map_or_else(
            |_| (0, BlockNumberOrTag::Number(0)), 
            |opt| match opt {
                Some(res) => {
                    match res.as_ref().try_into() {
                        Ok(bytes) => {
                            (u64::from_be_bytes(bytes), BlockNumberOrTag::Number(u64::from_be_bytes(bytes) + 1))
                        },
                        Err(_) => (0, BlockNumberOrTag::Number(0)),
                    }
                }
                None => (0, BlockNumberOrTag::Number(0)),
            }
        );
    
    println!("indexer_block_height {} {}", indexer_block_height, chain_id);

    let mut transaction_count = match provider.get_block_by_number(query_param, true).await {
        Ok(block) => block.unwrap().transactions.len(),
        Err(_) => 0
    };

    loop {
        let mut validator_max_height = match provider.get_block_number().await {
            Ok(ht) => ht,
            Err(_) => {
                sleep(time::Duration::from_millis(SLEEP)).await;
                continue;
            }
        };

        println!("validator_max_height {}", validator_max_height);

        while let Ok(current_block) = provider.get_block_by_number(query_param, true).await {
            if current_block.is_none() {
                break;
            }
            let current_block = current_block.unwrap(); 
            validator_max_height = std::cmp::max(validator_max_height, current_block.header.number);
            if indexer_block_height == 0 || indexer_block_height != validator_max_height
            {
                let running_transaction_count = current_block.transactions.len().saturating_add(transaction_count);
                println!("running_transaction_count {}", running_transaction_count);
                // save tx count
                let _ = db.insert(&format!("{}:{}", chain_id, current_block.header.number), &running_transaction_count.to_be_bytes());
                indexer_block_height = current_block.header.number;
                println!("indexer_block_height {}", indexer_block_height);
                // save latest height
                let _ = db.insert(&chain_id.to_be_bytes(), &current_block.header.number.to_be_bytes());
            }
            query_param = BlockNumberOrTag::Number(indexer_block_height.saturating_add(1));
            transaction_count = current_block.transactions.len();
            sleep(time::Duration::from_millis(SLEEP)).await;
        }
    }
}

const SLEEP: u64 = 10;
