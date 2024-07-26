use std::{io, sync::Arc};

use actix_web::{post, web, App, HttpServer, Responder};
use scout_interpreter::builder::InterpreterBuilder;
use tokio::sync::Mutex;

use crate::config::ConfigInputsHttp;

#[post("/crawl")]
async fn crawl(crawler: web::Data<Arc<Mutex<fantoccini::Client>>>) -> impl Responder {
    let f = &*crawler.get_ref().lock().await;
    format!("hello")
}

pub async fn start_http_consumer(
    config: &ConfigInputsHttp,
    crawler: Arc<Mutex<fantoccini::Client>>,
) -> Result<(), io::Error> {
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(crawler.clone()))
            .service(crawl)
    })
    .bind((config.addr.as_str(), config.port as u16))?
    .run()
    .await
}
