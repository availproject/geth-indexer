use alloy::providers::{Provider, ProviderBuilder, RootProvider};
use alloy::transports::http::Http;
use db::provider::InternalDataProvider;
use reqwest::Client;
use std::collections::BTreeMap;
use std::sync::Arc;

use crate::catchup::catch_up_blocks;
use crate::config::IndexerConfig;

pub struct Indexer {
    pub internal_provider: Arc<InternalDataProvider>,
    pub chain_ids: Vec<u64>,
    pub external_providers: BTreeMap<u64, ExternalProvider>,
}

impl Indexer {
    pub async fn new(config: IndexerConfig, internal_provider: Arc<InternalDataProvider>) -> Self {
        let mut external_providers = BTreeMap::new();
        let mut chain_ids = Vec::new();

        for endpoint in config.geth_endpoints.clone() {
            let provider = ProviderBuilder::new().on_http(endpoint.parse().unwrap());
            let chain_id = provider
                .get_chain_id()
                .await
                .expect("chain id should be readable");
            chain_ids.push(chain_id);
            external_providers.insert(chain_id, provider);
        }

        Self {
            internal_provider,
            external_providers,
            chain_ids,
        }
    }

    pub async fn run(&self) {
        for chain_id in &self.chain_ids {
            let internal_provider = self.internal_provider.clone();
            if let Some(external_provider) = self.external_providers.get(chain_id) {
                let external_provider = external_provider.clone();
                let chain_id = chain_id.clone();
                tokio::spawn(async move {
                    let _ = catch_up_blocks(internal_provider, external_provider, &chain_id).await;
                });
            }
        }
    }
}

pub type ExternalProvider = RootProvider<Http<Client>>;
