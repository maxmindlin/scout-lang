use std::fs;

use serde::Deserialize;

use crate::WorkerError;

#[derive(Debug, Default, Deserialize)]
pub struct Config {
    pub inputs: ConfigInputs,
}

#[derive(Debug, Default, Deserialize)]
pub struct ConfigInputs {
    pub http: Option<ConfigInputsHttp>,
}

#[derive(Debug, Deserialize)]
pub struct ConfigInputsHttp {
    pub addr: String,
    pub port: usize,
}

impl Config {
    pub fn load_file(path: Option<&str>) -> Result<Self, WorkerError> {
        let path = path.unwrap_or("scout.toml");
        let content =
            fs::read_to_string(path).map_err(|e| WorkerError::ConfigError(e.to_string()))?;
        toml::from_str(&content).map_err(|e| WorkerError::ConfigError(e.to_string()))
    }
}
