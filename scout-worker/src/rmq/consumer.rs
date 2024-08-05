use futures_lite::StreamExt;
use lapin::{
    options::{
        BasicAckOptions, BasicConsumeOptions, BasicNackOptions, ExchangeDeclareOptions,
        QueueDeclareOptions,
    },
    types::FieldTable,
    Channel, Connection, ConnectionProperties, ExchangeKind,
};
use scout_interpreter::{
    builder::{BuilderError, InterpreterBuilder},
    Interpreter, InterpreterError,
};
use std::{fmt::Display, fs, str};

use crate::{config::ConfigRMQ, models::incoming};

#[derive(Debug)]
pub enum ConsumerError {
    RabbitError,
    ScoutError,
}

pub struct Consumer {
    chann: Channel,
    queue: String,
    interpreter: Interpreter,
}

impl Consumer {
    pub async fn new(config: &ConfigRMQ) -> Result<Self, ConsumerError> {
        let conn = Connection::connect(&config.addr, ConnectionProperties::default()).await?;
        let chann = conn.create_channel().await?;
        let interpreter = InterpreterBuilder::default().build().await?;

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
            interpreter,
        })
    }

    async fn process(&mut self, data: &[u8]) -> Result<String, ConsumerError> {
        let raw = str::from_utf8(data).map_err(|_| ConsumerError::RabbitError)?;
        let incoming: incoming::Incoming =
            serde_json::from_str(raw).map_err(|_| ConsumerError::RabbitError)?;
        let content = fs::read_to_string(incoming.file).map_err(|_| ConsumerError::RabbitError)?;
        let _ = self.interpreter.eval(&content).await?;
        let res = self.interpreter.results();
        let payload = res.lock().await.to_json();
        self.interpreter.reset();
        Ok(payload)
    }

    pub async fn start(&mut self) -> Result<(), lapin::Error> {
        let mut consumer = self
            .chann
            .basic_consume(
                &self.queue,
                "scout-worker",
                BasicConsumeOptions::default(),
                FieldTable::default(),
            )
            .await?;

        while let Some(delivery) = consumer.next().await {
            if let Ok(delivery) = delivery {
                match self.process(&delivery.data).await {
                    Ok(res) => {
                        delivery.ack(BasicAckOptions::default()).await?;
                    }
                    Err(_) => {
                        delivery.nack(BasicNackOptions::default()).await?;
                    }
                }
            }
        }

        Ok(())
    }
}

impl From<lapin::Error> for ConsumerError {
    fn from(_value: lapin::Error) -> Self {
        Self::RabbitError
    }
}

impl From<InterpreterError> for ConsumerError {
    fn from(_value: InterpreterError) -> Self {
        Self::ScoutError
    }
}

impl From<BuilderError> for ConsumerError {
    fn from(_value: BuilderError) -> Self {
        Self::ScoutError
    }
}

impl Display for ConsumerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::RabbitError => write!(f, "consumer rabbit error"),
            Self::ScoutError => write!(f, "consumer scout error"),
        }
    }
}

impl std::error::Error for ConsumerError {}
