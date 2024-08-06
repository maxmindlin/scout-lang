use reqwest::Method;

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
        let req = self
            .client
            .request(self.method.clone(), &self.endpoint)
            .body(payload.to_owned())
            .build()?;
        self.client.execute(req).await?;

        Ok(())
    }
}
