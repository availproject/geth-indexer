use std::collections::BTreeMap;
use std::sync::Arc;
use alloy::providers::{Provider, ProviderBuilder, RootProvider};
use alloy::transports::http::Http;
use reqwest::Client;

use crate::config::IndexerConfig;
use crate::catchup::catch_up_blocks;

pub struct Indexer {
    pub db: Arc<sled::Db>,
    pub chain_ids: Vec<u64>,
    pub providers: BTreeMap<u64, NodeProvider>,
}

impl Indexer {
    pub async fn new(config: IndexerConfig, db: Arc<sled::Db>) -> Self {
        let mut providers = BTreeMap::new();
        let mut chain_ids = Vec::new();
        for endpoint in config.geth_endpoints.clone() {
            let provider = ProviderBuilder::new().on_http(endpoint.parse().unwrap());
            let chain_id = provider.get_chain_id().await.expect("");
            chain_ids.push(chain_id);
            providers.insert(chain_id, provider);
        }
        Self { db, providers, chain_ids }
    }

    pub async fn run(&self) {
        for chain_id in &self.chain_ids {
            let db = self.db.clone(); 
            if let Some(provider) = self.providers.get(chain_id) {
                let provider = provider.clone();
                let chain_id = chain_id.clone();
                tokio::spawn(async move {
                    let _ = catch_up_blocks(db, provider, &chain_id).await;
                });
            }
        }
    }
}


pub type NodeProvider = RootProvider<Http<Client>>;
