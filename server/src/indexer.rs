use alloy::{
    providers::{RootProvider, Provider},
    transports::http::Http,
};
use db::provider::InternalDataProvider;
use reqwest::Client;
use std::{collections::BTreeMap, sync::Arc, time::Duration};
use tokio::sync::Mutex;

use crate::{catchup::catch_up_blocks, config::IndexerConfig};

pub struct Indexer {
    pub chain_ids: Vec<u64>,
    pub internal_provider: Arc<InternalDataProvider>,
    pub external_providers: BTreeMap<u64, ExternalProvider>,
    pub inactive_providers: BTreeMap<String, ExternalProvider>,
    pub indexer_start_heights: BTreeMap<u64, Option<u64>>,
}

impl Indexer {
    pub async fn new(
        config: IndexerConfig,
        internal_provider: Arc<InternalDataProvider>,
        external_providers: BTreeMap<u64, ExternalProvider>,
        inactive_providers: BTreeMap<String, ExternalProvider>,
    ) -> Self {
        let mut chain_ids = Vec::new();
        let mut indexer_heights = BTreeMap::new();

        for (idx, (chain_id, _)) in external_providers.iter().enumerate() {
            let chain_id = *chain_id;
            let indexer_ht = if config.indexer_start_heights[idx] == -1 {
                None
            } else {
                Some(config.indexer_start_heights[idx].clone() as u64)
            };
            chain_ids.push(chain_id);
            indexer_heights.insert(chain_id, indexer_ht);
        }

        Self {
            internal_provider,
            external_providers,
            chain_ids,
            indexer_start_heights: indexer_heights,
            inactive_providers,
        }
    }

    pub async fn bootstrap(&self) {
        for chain_id in &self.chain_ids {
            let internal_provider = self.internal_provider.clone();
            if let Some(external_provider) = self.external_providers.get(chain_id) {
                let external_provider = external_provider.clone();
                let indexer_start_height = *self
                    .indexer_start_heights
                    .get(chain_id)
                    .expect("Irrecoverable Error: Start height should be present.");
                let chain_id = *chain_id;
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

        self.poll_inactive_providers().await;
    }

    pub async fn poll_inactive_providers(&self) {
        let inactive_providers = Arc::new(Mutex::new(self.inactive_providers.clone()));
        let internal_provider = self.internal_provider.clone();
    
        tokio::spawn(async move {
            loop {
                let internal_provider = internal_provider.clone();
                let mut to_remove = Vec::new();
    
                {
                    let providers = inactive_providers.lock().await;
                    for (endpoint, provider) in providers.iter() {
                        let chain_id = match provider.get_chain_id().await {
                            Ok(id) => id,
                            Err(_) => {
                                tracing::info!("Chain ID not readable — node still down for endpoint {endpoint}");
                                continue;
                            }
                        };
    
                        to_remove.push(endpoint.clone());
    
                        let provider = provider.clone();
                        let internal_provider = internal_provider.clone();
    
                        tokio::spawn(async move {
                            let _ = catch_up_blocks(
                                None,
                                internal_provider,
                                provider,
                                &chain_id,
                            )
                            .await;
                        });
                    }
                }
    
                let mut providers = inactive_providers.lock().await;
                for key in to_remove {
                    providers.remove(&key);
                }
    
                tokio::time::sleep(Duration::from_secs(120)).await;
            }
        });
    }
    
}

pub type ExternalProvider = RootProvider<Http<Client>>;
