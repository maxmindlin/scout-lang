use std::fs;

use serde::Deserialize;

use crate::WorkerError;

const DFEAULT_CONFIG_FILE: &str = "scout.toml";

#[derive(Debug, Default, Deserialize)]
pub struct Config {
    pub inputs: ConfigInputs,
    pub outputs: Option<ConfigOutputs>,
}

#[derive(Debug, Default, Deserialize)]
pub struct ConfigOutputs {
    pub rmq: Option<ConfigRMQ>,
    pub http: Option<ConfigOutputHttp>,
}

#[derive(Debug, Default, Deserialize)]
pub struct ConfigInputs {
    pub http: Option<ConfigInputHttp>,
    pub rmq: Option<ConfigRMQ>,
}

#[derive(Debug, Deserialize)]
pub struct ConfigInputHttp {
    pub addr: String,
    pub port: usize,
}

#[derive(Debug, Deserialize)]
pub enum OutputMethods {
    POST,
    PUT,
    PATCH,
}

#[derive(Debug, Deserialize)]
pub struct ConfigOutputHttp {
    pub endpoint: String,
    pub method: OutputMethods,
}

#[derive(Debug, Deserialize)]
pub struct ConfigRMQ {
    pub addr: String,
    pub queue: String,
    pub exchange: String,
    pub routing_key: String,
}

impl Config {
    pub fn load_file(path: Option<&str>) -> Result<Self, WorkerError> {
        let path = path.unwrap_or(DFEAULT_CONFIG_FILE);
        let content =
            fs::read_to_string(path).map_err(|e| WorkerError::ConfigError(e.to_string()))?;
        toml::from_str(&content).map_err(|e| WorkerError::ConfigError(e.to_string()))
    }
}

impl std::fmt::Display for OutputMethods {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::PATCH => write!(f, "PATCH"),
            Self::POST => write!(f, "POST"),
            Self::PUT => write!(f, "PUT"),
        }
    }
}
