use clap::Parser;
use serde_derive::{Deserialize, Serialize};
use std::{fs, path::PathBuf};

#[derive(Parser, Debug)]
pub struct CLIArguments {
    #[clap(long, value_parser)]
    pub config_path: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct IndexerConfig {
    pub listening_port: u16,
    pub geth_endpoints: Vec<String>,
    pub sled_path: PathBuf,
}

impl Default for IndexerConfig {
    fn default() -> Self {
        IndexerConfig {
            listening_port: 9090,
            geth_endpoints: vec!["http://139.59.46.36:22001".to_string(), "http://128.199.25.233:22001".to_string(), "http://139.59.30.161:22001".to_string()],
            sled_path: PathBuf::from("."),
        }
    }
}

pub(crate) fn load_config(config_path: &str) -> std::result::Result<IndexerConfig, String> {
    match fs::read_to_string(config_path) {
        Ok(file_str) => {
            let ret: IndexerConfig = match toml::from_str(&file_str) {
                Ok(r) => r,
                Err(_) => return Err("config.toml is not a proper toml file.".to_string()),
            };
            Ok(ret)
        }
        Err(_) => Ok(IndexerConfig::default()),
    }
}
