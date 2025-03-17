use clap::Parser;
use dotenvy::dotenv;
use tracing::Level;

mod config;
mod error;
mod routes;
mod server;

use crate::config::{load_config, CLIArguments};
use crate::server::Server;

mod catchup;
mod indexer;

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    dotenv().ok();

    tracing_subscriber::fmt().with_max_level(Level::INFO).init();

    let cli_args = CLIArguments::parse();
    let config_path = cli_args.config_path.unwrap_or(String::new());
    let config = load_config(&config_path).expect("Irrecoverable error: fail to load config.toml");

    Server::new(config).await?.start().await?;

    Ok(())
}
