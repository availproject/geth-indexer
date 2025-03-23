use alloy::providers::{Provider, ProviderBuilder};
use db::provider::InternalDataProvider;
use std::{collections::BTreeMap, sync::Arc};
use tracing::info;
use warp::Filter;

use crate::{
    config::IndexerConfig,
    error::handle_rejection,
    indexer::{ExternalProvider, Indexer},
    routes::*,
};

pub(crate) struct Server {
    config: IndexerConfig,
    external_providers: BTreeMap<u64, ExternalProvider>,
    internal_data_provider: Arc<InternalDataProvider>,
}

impl Server {
    pub async fn new(config: IndexerConfig) -> Result<Server, std::io::Error> {
        let mut external_providers = BTreeMap::new();
        for (_idx, endpoint) in config.geth_endpoints.clone().iter().enumerate() {
            let provider = ProviderBuilder::new().on_http(endpoint.parse().unwrap());
            let chain_id = match provider.get_chain_id().await {
                Ok(id) => id,
                Err(_) => {
                    info!("chain id should be readable");
                    continue;
                }
            };
            external_providers.insert(chain_id, provider);
        }

        Ok(Server {
            config,
            external_providers,
            internal_data_provider: Arc::new(InternalDataProvider::new().await?),
        })
    }

    pub async fn start(self) -> Result<(), std::io::Error> {
        let listening_port = self.config.listening_port.clone();
        let _ = Indexer::new(
            self.config,
            self.internal_data_provider.clone(),
            self.external_providers.clone(),
        )
        .await
        .run()
        .await;

        let warp_serve = warp::serve(
            index_route()
                .or(metrics(self.internal_data_provider.clone()))
                .or(transactions(
                    self.internal_data_provider.clone(),
                    self.external_providers.clone(),
                ))
                .recover(handle_rejection)
                .with(warp::cors().allow_any_origin()),
        );

        let (_, server) =
            warp_serve.bind_with_graceful_shutdown(([0, 0, 0, 0], listening_port), async move {
                tokio::signal::ctrl_c()
                    .await
                    .expect("failed to listen to shutdown signal");
            });

        server.await;

        Ok(())
    }
}
