use std::sync::Arc;

use clap::Parser;
use dotenvy::dotenv;

mod config;
mod error;
mod routes;
mod server;

use crate::config::{load_config, CLIArguments};
use crate::server::Server;

mod indexer;
mod catchup;

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    dotenv().ok();

    let cli_args = CLIArguments::parse();
    let config_path = cli_args.config_path.unwrap_or(String::new());
    let config = load_config(&config_path).expect("Irrecoverable error: fail to load config.toml");

    let db = Arc::new(sled::open(config.sled_path.clone()).expect("Failed to open sled db"));
    let db_instance_rpc = Arc::clone(&db);
    
    Server::new(config, db_instance_rpc).start().await?;

    Ok(())
}
