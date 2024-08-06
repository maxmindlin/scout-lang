use crate::{http, rmq};

pub enum Output {
    RMQ(rmq::producer::Producer),
    HTTP(http::sender::Sender),
}

impl Output {
    pub async fn send(&self, payload: &str) -> Result<(), Box<dyn std::error::Error>> {
        match self {
            Output::RMQ(p) => {
                p.send(payload).await?;
                Ok(())
            }
            Output::HTTP(sender) => {
                sender.send(payload).await?;
                Ok(())
            }
        }
    }
}
