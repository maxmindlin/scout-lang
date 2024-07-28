use std::sync::Arc;

use config::Config;
use scout_interpreter::{builder::InterpreterBuilder, Interpreter};
use tokio::sync::Mutex;

mod config;
mod http;
mod models;

pub enum WorkerError {
    ConfigError(String),
}

async fn start(config: Config, interpeter: Interpreter) {
    let inter_ptr = Arc::new(Mutex::new(interpeter));
    if let Some(http_config) = config.inputs.http {
        if let Err(e) = http::start_http_consumer(&http_config, inter_ptr.clone()).await {
            println!("{e:?}");
        }
    }
}

#[tokio::main]
async fn main() {
    let config = Config::load_file(None).unwrap_or_default();
    let interpreter = InterpreterBuilder::default()
        .build()
        .await
        .expect("error building interpreter");
    start(config, interpreter).await;
}
