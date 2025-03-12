use axum::{
    http::{header, Method},
    routing::get,
    Router,
};
use std::{env, sync::Arc};
use tokio::net::TcpListener;
use tower_http::{
    cors::{Any, CorsLayer},
    trace::{self as tower_trace, TraceLayer},
};
use tower_trace::{DefaultMakeSpan, DefaultOnRequest, DefaultOnResponse};
use tracing::Level;

use crate::{
    config::IndexerConfig,
    routes::{get_tps, health_check},
    indexer::Indexer,
};

pub(crate) struct Server {
    config: IndexerConfig,
    db: Arc<sled::Db>,

}

impl Server {
    pub fn new(config: IndexerConfig, db: Arc<sled::Db>) -> Server {
        Server { config, db }
    }

    pub async fn start(self) -> Result<(), std::io::Error> {
        let listening_port = self.config.listening_port.clone();
        let indexer = Indexer::new(self.config, self.db.clone()).await;        
        let db = self.db.clone();
        let app = Router::new()
            .route("/", get(health_check))
            .route("/tps/:chain_id", get(get_tps))
            .with_state(db)
            .layer(
                CorsLayer::new()
                    .allow_origin(Any)
                    .allow_headers(vec![header::CONTENT_TYPE])
                    .allow_methods([Method::POST]),
            )
            .layer(
                TraceLayer::new_for_http()
                    .make_span_with(DefaultMakeSpan::new().level(Level::INFO))
                    .on_request(DefaultOnRequest::new().level(Level::INFO))
                    .on_response(DefaultOnResponse::new().level(Level::INFO)),
            );

        let listener = TcpListener::bind((
            "0.0.0.0",
            env::var("PORT")
                .ok()
                .and_then(|x| x.parse().ok())
                .unwrap_or(listening_port),
        ))
        .await?;

        indexer.run().await;

        axum::serve(listener, app).await?;


        Ok(())
    }
}

