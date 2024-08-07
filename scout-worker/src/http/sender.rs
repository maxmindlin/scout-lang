use reqwest::Method;
use tracing::info;

use crate::config::OutputMethods;

pub struct Sender {
    client: reqwest::Client,
    method: Method,
    endpoint: String,
}

impl Sender {
    pub fn new(
        method: &OutputMethods,
        endpoint: String,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let client = reqwest::Client::new();
        let method = Method::from_bytes(method.to_string().as_bytes())?;
        Ok(Self {
            client,
            method,
            endpoint,
        })
    }

    pub async fn send(&self, payload: &str) -> Result<(), reqwest::Error> {
        info!("sending output to {} {}", self.method, self.endpoint);
        let req = self
            .client
            .request(self.method.clone(), &self.endpoint)
            .body(payload.to_owned())
            .build()?;
        let res = self.client.execute(req).await?;
        info!("received response code {}", res.status());
        Ok(())
    }
}
