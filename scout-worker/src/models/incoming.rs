use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Incoming {
    pub file: String,
}
