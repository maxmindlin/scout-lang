use get_port::Ops;

use crate::{
    browser::{AnyBrowserAdapter, Chrome, ChromeAdapter, Firefox, FirefoxAdapter},
    env::EnvPointer,
    eval::ScrapeResultsPtr,
    DriverProcess, EnvVars, Interpreter,
};

#[derive(Debug)]
pub enum BuilderError {
    BrowserStartup(String),
    EnvError(String),
    InvalidBrowser(String),
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
        let port = env_vars
            .port()
            .unwrap_or_else(|| get_port::tcp::TcpPort::any("127.0.0.1").unwrap() as usize);
        let browser_type = std::env::var("SCOUT_BROWSER")
            .unwrap_or_else(|_| "firefox".to_string())
            .to_lowercase();

        let adapter: Box<dyn AnyBrowserAdapter> = match browser_type.as_str() {
            "firefox" => Box::new(FirefoxAdapter::new(Firefox)),
            "chrome" => Box::new(ChromeAdapter::new(Chrome)),
            other => {
                return Err(BuilderError::InvalidBrowser(format!(
                    "unsupported browser: {other}. supported: firefox, chrome"
                )))
            }
        };

        adapter.start(port)?;
        let crawler = match self.crawler {
            Some(c) => c,
            None => adapter.connect(port, &env_vars).await?,
        };

        let interpreter = Interpreter::new(
            self.env.unwrap_or_default(),
            self.results.unwrap_or_default(),
            crawler,
            DriverProcess(adapter),
        );

        Ok(interpreter)
    }
}

impl std::fmt::Display for BuilderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BuilderError::BrowserStartup(e) => write!(f, "{}", e),
            BuilderError::EnvError(e) => write!(f, "{}", e),
            BuilderError::InvalidBrowser(e) => write!(f, "{}", e),
        }
    }
}

impl std::error::Error for BuilderError {}
