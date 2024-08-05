use std::{
    fs,
    io::{self, ErrorKind},
};

use actix_web::{get, http::StatusCode, post, web, App, HttpResponse, HttpServer, Responder};
use scout_interpreter::builder::InterpreterBuilder;

use crate::{config::ConfigInputsHttp, models::incoming};

#[post("/")]
async fn crawl(body: web::Json<incoming::Incoming>) -> impl Responder {
    match fs::read_to_string(&body.file) {
        Ok(content) => {
            let interpreter = InterpreterBuilder::default().build().await.unwrap();
            if let Err(e) = interpreter.eval(&content).await {
                return HttpResponse::build(StatusCode::INTERNAL_SERVER_ERROR)
                    .body(format!("interpreter error: {e:?}"));
            }
            let res = interpreter.results();
            let payload = res.lock().await.to_json();
            interpreter.close().await;

            HttpResponse::build(StatusCode::OK)
                .content_type("application/json")
                .body(payload)
        }
        Err(e) if e.kind() == ErrorKind::NotFound => HttpResponse::build(StatusCode::BAD_REQUEST)
            .body(format!("unknown file: {}", body.file)),
        Err(e) => HttpResponse::build(StatusCode::INTERNAL_SERVER_ERROR)
            .body(format!("error reading file: {e}")),
    }
}

#[get("/health")]
async fn health() -> impl Responder {
    "OK"
}

pub async fn start_http_consumer(config: &ConfigInputsHttp) -> Result<(), io::Error> {
    HttpServer::new(move || App::new().service(crawl).service(health))
        .bind((config.addr.as_str(), config.port as u16))?
        .run()
        .await
}
