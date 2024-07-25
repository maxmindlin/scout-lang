use crate::{env::EnvPointer, eval::ScrapeResultsPtr, EnvVars, Interpreter};

#[derive(Debug)]
pub enum BuilderError {
    BrowserStartup(String),
    EnvError(String),
}

#[derive(Default)]
pub struct InterpreterBuilder {
    env: Option<EnvPointer>,
    crawler: Option<fantoccini::Client>,
    results: Option<ScrapeResultsPtr>,
}

impl InterpreterBuilder {
    pub fn with_env(mut self, env: EnvPointer) -> Self {
        self.env = Some(env);
        self
    }

    pub fn with_crawler(mut self, crawler: fantoccini::Client) -> Self {
        self.crawler = Some(crawler);
        self
    }

    pub fn with_results(mut self, results: ScrapeResultsPtr) -> Self {
        self.results = Some(results);
        self
    }

    pub async fn build(self) -> Result<Interpreter, BuilderError> {
        let env_vars =
            envy::from_env::<EnvVars>().map_err(|e| BuilderError::EnvError(e.to_string()))?;
        let crawler = match self.crawler {
            Some(c) => Ok(c),
            None => new_crawler(&env_vars).await,
        }?;
        let interpreter = Interpreter::new(
            self.env.unwrap_or(EnvPointer::default()),
            self.results.unwrap_or(ScrapeResultsPtr::default()),
            crawler,
        );
        Ok(interpreter)
    }
}

async fn new_crawler(env_vars: &EnvVars) -> Result<fantoccini::Client, BuilderError> {
    let mut caps = serde_json::map::Map::new();
    if !env_vars.scout_debug {
        let opts = serde_json::json!({ "args": ["--headless"] });
        caps.insert("moz:firefoxOptions".into(), opts);
    }
    if let Some(proxy) = env_vars.scout_proxy.clone() {
        let opt = serde_json::json!({
            "proxyType": "manual",
            "httpProxy": proxy,
        });
        caps.insert("proxy".into(), opt);
    }
    let conn_url = format!("http://localhost:{}", env_vars.scout_port);
    let crawler = fantoccini::ClientBuilder::native()
        .capabilities(caps)
        .connect(&conn_url)
        .await
        .map_err(|e| BuilderError::BrowserStartup(e.to_string()))?;
    Ok(crawler)
}

impl std::fmt::Display for BuilderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BuilderError::BrowserStartup(e) => write!(f, "{}", e),
            BuilderError::EnvError(e) => write!(f, "{}", e),
        }
    }
}

impl std::error::Error for BuilderError {}
