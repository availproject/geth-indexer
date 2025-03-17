use alloy::eips::BlockNumberOrTag;
use alloy::primitives::U256;
use alloy::providers::Provider;
use alloy::rpc::types::Block;
use async_std::task::sleep;
use db::provider::InternalDataProvider;
use std::sync::Arc;
use std::time;

use crate::error::IndexerError;
use crate::indexer::ExternalProvider;

pub(crate) async fn catch_up_blocks(
    internal_provider: Arc<InternalDataProvider>,
    external_provider: ExternalProvider,
    chain_id: &u64,
) -> Result<(), IndexerError> {
    let (mut indexer_block_height, mut query_param) =
        match internal_provider.get_latest_height(chain_id).await {
            Ok(ht) => (ht, BlockNumberOrTag::Number(ht + 1)),
            Err(_) => (0, BlockNumberOrTag::Number(0)),
        };

    loop {
        let mut validator_max_height = match external_provider.get_block_number().await {
            Ok(ht) => ht,
            Err(_) => {
                sleep(time::Duration::from_millis(SLEEP)).await;
                continue;
            }
        };

        while let Ok(current_block) = external_provider
            .get_block_by_number(query_param, true)
            .await
        {
            if current_block.is_none() {
                break;
            }
            let current_block = current_block.unwrap();
            validator_max_height = std::cmp::max(validator_max_height, current_block.header.number);
            if indexer_block_height == 0 || indexer_block_height != validator_max_height {
                indexer_block_height = current_block.header.number;
                let (total_xfers, successful_xfers) =
                    match count_native_transfers(&current_block, &external_provider).await {
                        Ok((total_xfers, successful_xfers)) => (total_xfers, successful_xfers),
                        Err(_) => break,
                    };

                if let Ok(()) = internal_provider
                    .add_block(
                        chain_id,
                        current_block.header.timestamp as i64,
                        successful_xfers,
                        total_xfers,
                        current_block.transactions.len(),
                        current_block.header.number,
                    )
                    .await
                {
                } else {
                    break;
                }
            }
            query_param = BlockNumberOrTag::Number(indexer_block_height.saturating_add(1));
            sleep(time::Duration::from_millis(SLEEP)).await;
        }
    }
}

pub async fn count_native_transfers(
    block: &Block,
    external_provider: &ExternalProvider,
) -> Result<(u64, u64), IndexerError> {
    let mut total = 0;
    let mut failed = 0;

    for tx in block.transactions.txns() {
        if tx.value > U256::ZERO && tx.input.is_empty() && tx.to.is_some() {
            total += 1;
            let receipt = external_provider
                .get_transaction_receipt(tx.hash)
                .await
                .map_err(|e| IndexerError::ProviderError(e.to_string()))?;
            let is_failed = receipt.map_or(false, |r| r.status() == false);
            if is_failed {
                failed += 1;
            }
        }
    }

    Ok((total, failed))
}

const SLEEP: u64 = 10;
