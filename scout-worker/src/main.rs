use config::Config;

mod config;
mod http;

pub enum WorkerError {
    ConfigError(String),
}

async fn start(config: Config) {
    if let Some(http_config) = config.inputs.http {
        if let Err(e) = http::start_http_consumer(&http_config).await {
            println!("{e:?}");
        }
    }
}

#[tokio::main]
async fn main() {
    let config = Config::load_file(None).unwrap_or_default();
    println!("{config:?}");
    start(config).await;
}
