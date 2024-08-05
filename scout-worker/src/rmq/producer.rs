use std::fmt::Display;

use lapin::{
    options::{ExchangeDeclareOptions, QueueDeclareOptions},
    types::FieldTable,
    Channel, Connection, ConnectionProperties, ExchangeKind,
};

use crate::config::ConfigRMQ;

#[derive(Debug)]
pub enum ProducerError {
    RabbitError,
}

pub struct Producer {
    chann: Channel,
    queue: String,
}

impl Producer {
    pub async fn new(config: &ConfigRMQ) -> Result<Self, ProducerError> {
        let conn = Connection::connect(&config.addr, ConnectionProperties::default()).await?;
        let chann = conn.create_channel().await?;

        chann
            .exchange_declare(
                &config.exchange,
                ExchangeKind::Topic,
                ExchangeDeclareOptions::default(),
                FieldTable::default(),
            )
            .await?;

        let _ = chann
            .queue_declare(
                &config.queue,
                QueueDeclareOptions::default(),
                FieldTable::default(),
            )
            .await?;

        Ok(Self {
            chann,
            queue: config.queue.clone(),
        })
    }

    pub async fn send(&self, payload: String) -> Result<(), ProducerError> {
        unimplemented!()
    }
}

impl From<lapin::Error> for ProducerError {
    fn from(_value: lapin::Error) -> Self {
        Self::RabbitError
    }
}

impl Display for ProducerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::RabbitError => write!(f, "rmq error"),
        }
    }
}

impl std::error::Error for ProducerError {}
