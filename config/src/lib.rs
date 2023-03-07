use miette::Diagnostic;
use serde::{Deserialize, Serialize};
use std::{
    fs::read_to_string,
    net::IpAddr,
    path::{Path, PathBuf},
};
use thiserror::Error;

#[derive(Debug, Error, Diagnostic)]
pub enum Error {
    #[error(transparent)]
    IOError(#[from] std::io::Error),

    #[error(transparent)]
    TomlError(#[from] toml::de::Error),
}

type Result<T> = miette::Result<T, Error>;

const ETC_DIR_PATH: &str = "/etc/opc";
const CONFIG_PATH: &str = "/etc/opc";
const VPC_DB_PATH: &str = "/etc/opc/vpcs/db";
const VPC_SEARCH_FILE_PATH: &str = "/etc/opc/vpcs/search";
const CLOUD_CONFIG_FILE_PATH: &str = "/etc/opc/config.toml";

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub etc_dir: PathBuf,
    pub config_dir: PathBuf,
    pub vpc_db_path: PathBuf,
    pub vpc_search_path: PathBuf,
    pub listen_ip: Option<IpAddr>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            etc_dir: PathBuf::from(ETC_DIR_PATH),
            config_dir: PathBuf::from(CONFIG_PATH),
            vpc_db_path: PathBuf::from(VPC_DB_PATH),
            vpc_search_path: PathBuf::from(VPC_SEARCH_FILE_PATH),
            listen_ip: None,
        }
    }
}

pub fn open() -> Result<Config> {
    let config_path = Path::new(CLOUD_CONFIG_FILE_PATH);
    if config_path.exists() {
        let content = read_to_string(&config_path)?;
        let cfg: Config = toml::from_str(&content)?;
        Ok(cfg)
    } else {
        Ok(Config::default())
    }
}
