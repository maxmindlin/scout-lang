use std::sync::Arc;

use config::Config;
use http::sender::Sender;
use output::Output;
use rmq::producer::Producer;
use tracing::error;

mod config;
mod http;
mod models;
mod output;
mod rmq;

pub enum WorkerError {
    ConfigError(String),
}

async fn start(config: Config) -> Result<(), Box<dyn std::error::Error>> {
    let mut outputs = Vec::new();
    if let Some(outputs_config) = config.outputs {
        if let Some(rmq) = outputs_config.rmq {
            let rmq_out = Producer::new(&rmq).await?;
            outputs.push(Output::RMQ(rmq_out));
        }

        if let Some(http) = outputs_config.http {
            let http_out = Sender::new(&http.method, http.endpoint)?;
            outputs.push(Output::HTTP(http_out));
        }
    }

    let aoutputs = Arc::new(outputs);
    if let Some(http_config) = config.inputs.http {
        http::server::start_http_consumer(&http_config, aoutputs.clone()).await?;
    } else if let Some(rmq_config) = config.inputs.rmq {
        rmq::consumer::Consumer::new(&rmq_config, aoutputs.clone())
            .await?
            .start()
            .await?;
    }

    Ok(())
}

#[tokio::main]
async fn main() {
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "info");
    }

    let config = Config::load_file(None).unwrap_or_default();
    if let Err(e) = start(config).await {
        error!("error starting worker: {e}");
    }
}
