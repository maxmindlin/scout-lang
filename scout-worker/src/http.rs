use std::{
    fs,
    io::{self, ErrorKind},
    sync::Arc,
};

use actix_web::{get, http::StatusCode, post, web, App, HttpResponse, HttpServer, Responder};
use scout_interpreter::Interpreter;
use tokio::sync::Mutex;

use crate::{config::ConfigInputsHttp, models::incoming};

#[post("/crawl")]
async fn crawl(
    body: web::Json<incoming::Incoming>,
    interpreter: web::Data<Arc<Mutex<Interpreter>>>,
) -> impl Responder {
    match fs::read_to_string(&body.file) {
        Ok(content) => {
            let interpreter = &mut *interpreter.get_ref().lock().await;

            if let Err(e) = interpreter.eval(&content).await {
                interpreter.reset();
                return HttpResponse::build(StatusCode::INTERNAL_SERVER_ERROR)
                    .body(format!("interpreter error: {e:?}"));
            }
            let res = interpreter.results();
            let payload = res.lock().await.to_json();

            interpreter.reset();
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

pub async fn start_http_consumer(
    config: &ConfigInputsHttp,
    interpeter: Arc<Mutex<Interpreter>>,
) -> Result<(), io::Error> {
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(interpeter.clone()))
            .service(crawl)
            .service(health)
    })
    .bind((config.addr.as_str(), config.port as u16))?
    .run()
    .await
}
