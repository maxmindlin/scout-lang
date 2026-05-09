use crate::builder::BuilderError;
use crate::EnvVars;
use fantoccini::Client;
use std::process::{Child, Command};
use std::sync::Mutex;

// ---------------------------------------------------------------------------
// Browser driver configuration — the only thing that varies per browser
// ---------------------------------------------------------------------------

pub trait BrowserDriver: Send + Sync {
    fn driver_binary(&self) -> &'static str;
    fn capabilities(
        &self,
        debug: bool,
        proxy: Option<&str>,
    ) -> serde_json::Map<String, serde_json::Value>;
}

pub struct Firefox;
pub struct Chrome;

impl BrowserDriver for Firefox {
    fn driver_binary(&self) -> &'static str {
        "geckodriver"
    }

    fn capabilities(
        &self,
        debug: bool,
        proxy: Option<&str>,
    ) -> serde_json::Map<String, serde_json::Value> {
        let mut caps = serde_json::Map::new();

        if !debug {
            caps.insert(
                "moz:firefoxOptions".into(),
                serde_json::json!({ "args": ["--headless"] }),
            );
        }
        if let Some(p) = proxy {
            caps.insert("proxy".into(), proxy_cap(p));
        }
        caps
    }
}

impl BrowserDriver for Chrome {
    fn driver_binary(&self) -> &'static str {
        "chromedriver"
    }

    fn capabilities(
        &self,
        debug: bool,
        proxy: Option<&str>,
    ) -> serde_json::Map<String, serde_json::Value> {
        let mut caps = serde_json::Map::new();

        let args: Vec<&str> = if debug { vec![] } else { vec!["--headless"] };
        caps.insert(
            "goog:chromeOptions".into(),
            serde_json::json!({ "args": args }),
        );
        if let Some(p) = proxy {
            caps.insert("proxy".into(), proxy_cap(p));
        }
        caps
    }
}

fn proxy_cap(proxy: &str) -> serde_json::Value {
    serde_json::json!({
        "proxyType": "manual",
        "httpProxy": proxy,
    })
}

// ---------------------------------------------------------------------------
// Single adapter — parameterised by BrowserDriver
// ---------------------------------------------------------------------------

pub struct BrowserAdapter<D: BrowserDriver> {
    driver: D,
    process: Mutex<Option<Child>>,
}

impl<D: BrowserDriver> BrowserAdapter<D> {
    pub fn new(driver: D) -> Self {
        Self {
            driver,
            process: Mutex::new(None),
        }
    }

    pub fn start(&self, port: usize) -> Result<(), BuilderError> {
        let mut guard = self.lock()?;

        if guard.is_some() {
            return Err(BuilderError::BrowserStartup(
                "driver is already running".into(),
            ));
        }

        let binary = self.driver.driver_binary();
        let child = Command::new(binary)
            .arg("--port")
            .arg(port.to_string())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn()
            .map_err(|e| {
                BuilderError::BrowserStartup(format!(
                    "failed to start {binary}: {e} — is it installed and on PATH?"
                ))
            })?;

        std::thread::sleep(std::time::Duration::from_millis(50));
        *guard = Some(child);
        Ok(())
    }

    pub async fn connect(&self, port: usize, env_vars: &EnvVars) -> Result<Client, BuilderError> {
        let caps = self
            .driver
            .capabilities(env_vars.scout_debug, env_vars.scout_proxy.as_deref());

        fantoccini::ClientBuilder::native()
            .capabilities(caps)
            .connect(&format!("http://localhost:{port}"))
            .await
            .map_err(|e| BuilderError::BrowserStartup(e.to_string()))
    }

    pub fn close(&self) -> Result<(), BuilderError> {
        if let Some(mut child) = self.lock()?.take() {
            child
                .kill()
                .map_err(|e| BuilderError::BrowserStartup(e.to_string()))?;
        }
        Ok(())
    }

    fn lock(&self) -> Result<std::sync::MutexGuard<'_, Option<Child>>, BuilderError> {
        self.process
            .lock()
            .map_err(|e| BuilderError::BrowserStartup(format!("mutex poisoned: {e}")))
    }
}

// ---------------------------------------------------------------------------
// Type aliases + dyn-compatible wrapper — supports the intended usage
// ---------------------------------------------------------------------------

pub type FirefoxAdapter = BrowserAdapter<Firefox>;
pub type ChromeAdapter = BrowserAdapter<Chrome>;

/// Object-safe façade used at the call site.
pub trait AnyBrowserAdapter: Send + Sync {
    fn start(&self, port: usize) -> Result<(), BuilderError>;
    fn connect<'a>(
        &'a self,
        port: usize,
        env_vars: &'a EnvVars,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<Client, BuilderError>> + Send + 'a>,
    >;
    fn close(&self) -> Result<(), BuilderError>;
}

impl<D: BrowserDriver + Send + Sync> AnyBrowserAdapter for BrowserAdapter<D> {
    fn start(&self, port: usize) -> Result<(), BuilderError> {
        self.start(port)
    }

    fn connect<'a>(
        &'a self,
        port: usize,
        env_vars: &'a EnvVars,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<Client, BuilderError>> + Send + 'a>,
    > {
        Box::pin(self.connect(port, env_vars))
    }

    fn close(&self) -> Result<(), BuilderError> {
        self.close()
    }
}
