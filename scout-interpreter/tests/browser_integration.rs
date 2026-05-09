use std::process::Command;
use std::time::Duration;

use get_port::Ops;
use scout_interpreter::browser::{BrowserDriver, Chrome, ChromeAdapter, Firefox, FirefoxAdapter};
use scout_interpreter::builder::BuilderError;
use scout_interpreter::EnvVars;

// Helper function to check if a driver binary is available
fn is_driver_available(driver: &str) -> bool {
    Command::new("which")
        .arg(driver)
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

// Helper function to check if Firefox is available
fn is_firefox_available() -> bool {
    Command::new("which")
        .arg("firefox")
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

// Helper function to check if Chrome is available
fn is_chrome_available() -> bool {
    // Try both 'google-chrome' and 'chromium-browser'
    Command::new("which")
        .arg("google-chrome")
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
        || Command::new("which")
            .arg("chromium-browser")
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
}

#[cfg(test)]
mod browser_integration_tests {
    use super::*;
    use get_port::tcp::TcpPort;

    async fn test_browser_adapter<D: BrowserDriver + Send + Sync + 'static>(
        adapter: scout_interpreter::browser::BrowserAdapter<D>,
        _driver_name: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let port = TcpPort::any("127.0.0.1").unwrap() as usize;
        // Start the browser driver
        adapter.start(port)?;
        tokio::time::sleep(Duration::from_millis(100)).await;

        // Create test environment variables by setting env vars temporarily
        std::env::set_var("SCOUT_DEBUG", "false"); // Run in headless mode for testing
        std::env::set_var("SCOUT_PORT", port.to_string());
        std::env::remove_var("SCOUT_PROXY");

        let env_vars: EnvVars = envy::from_env().expect("Failed to parse env vars");

        // Clean up env vars
        std::env::remove_var("SCOUT_DEBUG");
        std::env::remove_var("SCOUT_PORT");

        // Connect to the browser
        let client = adapter.connect(port, &env_vars).await?;

        // Test basic browser operations
        client.goto("https://httpbin.org/html").await?;
        tokio::time::sleep(Duration::from_millis(500)).await;

        // Verify we can get the current URL
        let current_url = client.current_url().await?;
        assert!(current_url.to_string().contains("httpbin.org"));

        // Test finding an element
        let element = client.find(fantoccini::Locator::Css("h1")).await?;
        let text = element.text().await?;
        assert!(!text.is_empty());

        // Close the browser connection
        client.close().await?;
        // Close the driver
        adapter.close()?;

        Ok(())
    }

    #[tokio::test]
    async fn test_firefox_adapter_integration() {
        // Skip test if Firefox or geckodriver is not available
        if !is_firefox_available() || !is_driver_available("geckodriver") {
            println!("Skipping Firefox integration test: Firefox or geckodriver not available");
            return;
        }

        let adapter = FirefoxAdapter::new(Firefox);

        match test_browser_adapter(adapter, "Firefox").await {
            Ok(()) => println!("Firefox integration test passed"),
            Err(e) => panic!("Firefox integration test failed: {}", e),
        }
    }

    #[tokio::test]
    async fn test_chrome_adapter_integration() {
        // Skip test if Chrome/Chromium or chromedriver is not available
        if !is_chrome_available() || !is_driver_available("chromedriver") {
            println!(
                "Skipping Chrome integration test: Chrome/Chromium or chromedriver not available"
            );
            return;
        }

        let adapter = ChromeAdapter::new(Chrome);

        match test_browser_adapter(adapter, "Chrome").await {
            Ok(()) => println!("Chrome integration test passed"),
            Err(e) => panic!("Chrome integration test failed: {}", e),
        }
    }

    #[tokio::test]
    async fn test_browser_driver_start_stop() {
        // Test that we can start and stop a browser driver without connecting
        if !is_firefox_available() || !is_driver_available("geckodriver") {
            println!("Skipping Firefox start/stop test: Firefox or geckodriver not available");
            return;
        }

        let adapter = FirefoxAdapter::new(Firefox);
        let port = TcpPort::any("127.0.0.1").unwrap() as usize;

        // Test starting
        adapter.start(port).expect("Failed to start Firefox driver");

        // Give it a moment to start
        tokio::time::sleep(Duration::from_millis(100)).await;

        // Test stopping
        adapter.close().expect("Failed to stop Firefox driver");
    }

    #[tokio::test]
    async fn test_invalid_driver_start() {
        // Test error handling when driver binary doesn't exist
        struct InvalidDriver;
        impl BrowserDriver for InvalidDriver {
            fn driver_binary(&self) -> &'static str {
                "nonexistent_driver_binary"
            }

            fn capabilities(
                &self,
                _debug: bool,
                _proxy: Option<&str>,
            ) -> serde_json::Map<String, serde_json::Value> {
                serde_json::Map::new()
            }
        }

        let adapter = scout_interpreter::browser::BrowserAdapter::new(InvalidDriver);
        let port = TcpPort::any("127.0.0.1").unwrap() as usize;

        let result = adapter.start(port);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            BuilderError::BrowserStartup(_)
        ));
    }

    #[tokio::test]
    async fn test_double_start_prevention() {
        // Test that starting an already started driver returns an error
        if !is_firefox_available() || !is_driver_available("geckodriver") {
            println!("Skipping double start test: Firefox or geckodriver not available");
            return;
        }

        let adapter = FirefoxAdapter::new(Firefox);
        let port = TcpPort::any("127.0.0.1").unwrap() as usize;

        // First start should succeed
        adapter.start(port).expect("First start should succeed");

        // Give it a moment
        tokio::time::sleep(Duration::from_millis(50)).await;

        // Second start should fail
        let result = adapter.start(port);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            BuilderError::BrowserStartup(_)
        ));

        adapter.close().expect("Cleanup should succeed");
    }
}
