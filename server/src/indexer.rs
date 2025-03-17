use alloy::providers::{Provider, ProviderBuilder, RootProvider};
use alloy::transports::http::Http;
use db::provider::InternalDataProvider;
use reqwest::Client;
use std::collections::BTreeMap;
use std::sync::Arc;
use tracing::info;

use crate::catchup::catch_up_blocks;
use crate::config::IndexerConfig;

pub struct Indexer {
    pub chain_ids: Vec<u64>,
    pub internal_provider: Arc<InternalDataProvider>,
    pub external_providers: BTreeMap<u64, ExternalProvider>,
    pub indexer_start_heights: BTreeMap<u64, Option<u64>>,
}

impl Indexer {
    pub async fn new(config: IndexerConfig, internal_provider: Arc<InternalDataProvider>) -> Self {
        let mut external_providers = BTreeMap::new();
        let mut chain_ids = Vec::new();
        let mut indexer_heights = BTreeMap::new();

        for (idx, endpoint) in config.geth_endpoints.clone().iter().enumerate() {
            let provider = ProviderBuilder::new().on_http(endpoint.parse().unwrap());
            let chain_id = match provider.get_chain_id().await {
                Ok(id) => id,
                Err(_) => {
                    info!("chain id should be readable");
                    continue;
                }
            };
            let indexer_ht = if config.indexer_start_heights[idx] == -1 {
                None
            } else {
                Some(config.indexer_start_heights[idx].clone() as u64)
            };
            chain_ids.push(chain_id);
            external_providers.insert(chain_id, provider);
            indexer_heights.insert(chain_id, indexer_ht);
        }

        Self {
            internal_provider,
            external_providers,
            chain_ids,
            indexer_start_heights: indexer_heights,
        }
    }

    pub async fn run(&self) {
        for chain_id in &self.chain_ids {
            let internal_provider = self.internal_provider.clone();
            if let Some(external_provider) = self.external_providers.get(chain_id) {
                let external_provider = external_provider.clone();
                let indexer_start_height = *self
                    .indexer_start_heights
                    .get(chain_id)
                    .expect("Irrecoverable Error: Chain ID should be present.");
                let chain_id = chain_id.clone();
                tokio::spawn(async move {
                    let _ = catch_up_blocks(
                        indexer_start_height,
                        internal_provider,
                        external_provider,
                        &chain_id,
                    )
                    .await;
                });
            }
        }
    }
}

pub type ExternalProvider = RootProvider<Http<Client>>;
