use clap::Parser;
use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Parser, Debug)]
pub struct CLIArguments {
    #[clap(long, value_parser)]
    pub config_path: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct IndexerConfig {
    pub listening_port: u16,
    pub geth_endpoints: Vec<String>,
    pub indexer_start_heights: Vec<i64>,
}

impl Default for IndexerConfig {
    fn default() -> Self {
        IndexerConfig {
            listening_port: 9090,
            geth_endpoints: vec!["http://139.59.46.36:22001".to_string()],
            indexer_start_heights: vec![438200],
        }
    }
}

pub(crate) fn load_config(config_path: &str) -> std::result::Result<IndexerConfig, String> {
    match fs::read_to_string(config_path) {
        Ok(file_str) => {
            let ret: IndexerConfig = match toml::from_str(&file_str) {
                Ok(r) => r,
                Err(e) => {
                    println!("error...loading default config {}", e);
                    IndexerConfig::default()
                }
            };
            Ok(ret)
        }
        Err(_) => Ok(IndexerConfig::default()),
    }
}
