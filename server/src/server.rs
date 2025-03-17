use db::provider::InternalDataProvider;
use std::sync::Arc;
use warp::Filter;

use crate::{config::IndexerConfig, error::handle_rejection, indexer::Indexer, routes::*};

pub(crate) struct Server {
    config: IndexerConfig,
    internal_data_provider: Arc<InternalDataProvider>,
}

impl Server {
    pub async fn new(config: IndexerConfig) -> Result<Server, std::io::Error> {
        Ok(Server {
            config,
            internal_data_provider: Arc::new(InternalDataProvider::new().await?),
        })
    }

    pub async fn start(self) -> Result<(), std::io::Error> {
        let listening_port = self.config.listening_port.clone();
        let indexer = Indexer::new(self.config, self.internal_data_provider.clone()).await;

        tokio::spawn(async move {
            let warp_serve = warp::serve(
                index_route()
                    .or(metrics(self.internal_data_provider.clone()))
                    .recover(handle_rejection)
                    .with(warp::cors().allow_any_origin()),
            );

            let (_, server) = warp_serve.bind_with_graceful_shutdown(
                ([0, 0, 0, 0], listening_port),
                async move {
                    tokio::signal::ctrl_c()
                        .await
                        .expect("failed to listen to shutdown signal");
                },
            );

            server.await;
        });

        indexer.run().await;

        Ok(())
    }
}
