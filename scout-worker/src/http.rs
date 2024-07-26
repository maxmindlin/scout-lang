use std::io;

use actix_web::{post, App, HttpServer, Responder};

use crate::config::ConfigInputsHttp;

#[post("/crawl")]
async fn file_process() -> impl Responder {
    format!("hello")
}

pub async fn start_http_consumer(config: &ConfigInputsHttp) -> Result<(), io::Error> {
    HttpServer::new(|| App::new().service(file_process))
        .bind((config.addr.as_str(), config.port as u16))?
        .run()
        .await
}
