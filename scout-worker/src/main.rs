use config::Config;
use rmq::producer::Producer;

mod config;
mod http;
mod models;
mod rmq;

pub enum WorkerError {
    ConfigError(String),
}

pub enum Output {
    RMQ(rmq::producer::Producer),
}

impl Output {
    pub async fn send(&self, payload: String) -> Result<(), Box<dyn std::error::Error>> {
        match self {
            Output::RMQ(p) => {
                p.send(payload).await?;
                Ok(())
            }
        }
    }
}

async fn start(config: Config) -> Result<(), Box<dyn std::error::Error>> {
    let mut outputs = Vec::new();
    if let Some(outputs_config) = config.outputs {
        if let Some(rmq) = outputs_config.rmq {
            let rmq_out = Producer::new(&rmq).await?;
            outputs.push(Output::RMQ(rmq_out));
        }
    }

    if let Some(http_config) = config.inputs.http {
        http::start_http_consumer(&http_config).await?;
    } else if let Some(rmq_config) = config.inputs.rmq {
        let mut consumer = rmq::consumer::Consumer::new(&rmq_config).await?;
        consumer.start().await.expect("error starting consumer");
    }

    Ok(())
}

#[tokio::main]
async fn main() {
    let config = Config::load_file(None).unwrap_or_default();
    if let Err(e) = start(config).await {
        println!("error starting worker: {e}")
    }
}
