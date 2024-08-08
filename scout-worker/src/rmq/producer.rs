use std::fmt::Display;

use lapin::{
    options::{BasicPublishOptions, ExchangeDeclareOptions, QueueDeclareOptions},
    types::FieldTable,
    BasicProperties, Channel, Connection, ConnectionProperties, ExchangeKind,
};
use tracing::info;

use crate::config::ConfigRMQ;

#[derive(Debug)]
pub enum ProducerError {
    RabbitError,
}

pub struct Producer {
    chann: Channel,
    exchange: String,
    out_key: String,
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
            exchange: config.exchange.clone(),
            out_key: config.routing_key.clone(),
        })
    }

    pub async fn send(&self, payload: &str) -> Result<(), ProducerError> {
        info!("publishing message to {}", self.out_key);
        self.chann
            .basic_publish(
                &self.exchange,
                &self.out_key,
                BasicPublishOptions::default(),
                &payload.as_bytes(),
                BasicProperties::default(),
            )
            .await?;
        Ok(())
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
