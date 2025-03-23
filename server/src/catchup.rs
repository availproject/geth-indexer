use alloy::eips::BlockNumberOrTag;
use alloy::providers::Provider;
use alloy::rpc::types::Block;
use async_std::task::sleep;
use db::parse_logs;
use db::provider::InternalDataProvider;
use db::ToHexString;
use futures::stream::{FuturesUnordered, StreamExt};
use std::sync::Arc;
use std::time;
use tokio::task;
use tracing::info;

use crate::error::IndexerError;
use crate::indexer::ExternalProvider;

pub(crate) async fn catch_up_blocks(
    indexer_start_height: Option<u64>,
    internal_provider: Arc<InternalDataProvider>,
    external_provider: ExternalProvider,
    chain_id: &u64,
) -> Result<(), IndexerError> {
    let (mut indexer_block_height, mut query_param) = if let Some(ht) = indexer_start_height {
        (ht, BlockNumberOrTag::Number(ht + 1))
    } else {
        match external_provider.get_block_number().await {
            Ok(ht) => (ht, BlockNumberOrTag::Number(ht + 1)),
            Err(_) => (0, BlockNumberOrTag::Number(0)),
        }
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
                continue;
            }
            let current_block = current_block.unwrap();
            validator_max_height = std::cmp::max(validator_max_height, current_block.header.number);
            if indexer_block_height == 0 || indexer_block_height != validator_max_height {
                indexer_block_height = current_block.header.number;
                let (total_xfers, failed_xfers, total_native_transfers, total_x_chain_transfers) =
                    match process_block(
                        &current_block,
                        chain_id,
                        &external_provider,
                        internal_provider.clone(),
                    )
                    .await
                    {
                        Ok((
                            total_xfers,
                            failed_xfers,
                            total_native_transfers,
                            total_x_chain_transfers,
                        )) => (
                            total_xfers,
                            failed_xfers,
                            total_native_transfers,
                            total_x_chain_transfers,
                        ),
                        Err(_) => {
                            break;
                        }
                    };

                info!(
                    "current height {} validator height {}, total_xfers {} failed_xfers {} native_txns {}. x_chain_xfers {}, chain id {} ",
                    current_block.header.number, validator_max_height, total_xfers, failed_xfers, total_native_transfers, total_x_chain_transfers, chain_id,
                );

                if let Ok(()) = internal_provider
                    .add_block(
                        chain_id,
                        current_block.header.timestamp as i64,
                        total_xfers.saturating_sub(failed_xfers),
                        total_xfers,
                        total_native_transfers,
                        total_x_chain_transfers,
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

pub async fn process_block(
    block: &Block,
    chain_id: &u64,
    external_provider: &ExternalProvider,
    internal_provider: Arc<InternalDataProvider>,
) -> Result<(u64, u64, u64, u64), IndexerError> {
    let mut total = 0;
    let mut failed = 0;
    let mut total_native_transfers = 0;
    let mut total_x_chain_transfers = 0;
    let transactions: Vec<_> = block.transactions.txns().cloned().collect();
    let tasks: FuturesUnordered<_> = transactions
        .iter()
        .map(|tx| {
            let provider = external_provider.clone();
            let tx = tx.clone();
            task::spawn(async move {
                if let Some(receipt) = provider
                    .get_transaction_receipt(tx.hash)
                    .await
                    .ok()
                    .flatten()
                {
                    let is_failed = !receipt.status();
                    let is_transfer = (tx.input.is_empty() || tx.input.to_hex_string() == "0x")
                        && tx.to.is_some();
                    let (_, xtps) = parse_logs(&receipt);
                    Some((1, is_failed as u64, is_transfer as u64, xtps as u64))
                } else {
                    None
                }
            })
        })
        .collect();

    let results = tasks.collect::<Vec<_>>().await;
    for result in results {
        if let Ok(Some((t, f, native_xfers, cross_chain_xfers))) = result {
            total += t;
            failed += f;
            total_native_transfers += native_xfers;
            total_x_chain_transfers += cross_chain_xfers;
        }
    }

    let chain_id = chain_id.clone();
    tokio::spawn(async move {
        if let Err(e) = internal_provider
            .add_txns(chain_id, transactions.len(), transactions)
            .await
        {
            tracing::error!("{}", e.to_string());
        }
    });

    Ok((
        total,
        failed,
        total_native_transfers,
        total_x_chain_transfers,
    ))
}

const SLEEP: u64 = 10;
