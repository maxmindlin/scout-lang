# Browser Setup Guide

This guide explains how to set up browsers and drivers for running Scout scripts that require browser automation.

## Supported Browsers

Scout currently supports:
- **Firefox** (recommended for development)
- **Chrome/Chromium** (requires additional setup)

## Firefox Setup (Recommended)

### macOS
```bash
# Using Homebrew (recommended)
brew install geckodriver

# Verify installation
geckodriver --version
which geckodriver
```

### Linux
```bash
# Ubuntu/Debian
sudo apt-get update
sudo apt-get install firefox-geckodriver

# Or manual installation
wget https://github.com/mozilla/geckodriver/releases/download/v0.34.0/geckodriver-v0.34.0-linux64.tar.gz
tar -xzf geckodriver-v0.34.0-linux64.tar.gz
sudo mv geckodriver /usr/local/bin/
sudo chmod +x /usr/local/bin/geckodriver
```

### Windows
```powershell
# Using Chocolatey
choco install selenium-gecko-driver

# Or download from: https://github.com/mozilla/geckodriver/releases
# Extract geckodriver.exe to a folder in PATH
```

## Chrome Setup

### ChromeDriver Compatibility

ChromeDriver must match your Chrome browser version:

| Chrome Version | ChromeDriver Version |
|----------------|---------------------|
| Chrome 147.x   | ChromeDriver 147.x  |
| Chrome 148.x   | ChromeDriver 148.x  |
| Latest         | Check [Chrome for Testing](https://googlechromelabs.github.io/chrome-for-testing/) |

### macOS Setup

#### Option 1: Automated Setup (Recommended)
```bash
# Check your Chrome version
"/Applications/Google Chrome.app/Contents/MacOS/Google Chrome" --version

# Download matching ChromeDriver (example for Chrome 147)
curl -L -o chromedriver.zip "https://storage.googleapis.com/chrome-for-testing-public/147.0.7727.117/mac-x64/chromedriver-mac-x64.zip"
unzip chromedriver.zip
sudo cp chromedriver-mac-x64/chromedriver /usr/local/bin/
chmod +x /usr/local/bin/chromedriver

# Verify installation
chromedriver --version
```

#### Option 2: Using Homebrew
```bash
# Install ChromeDriver via Homebrew
brew install --cask chromedriver

# Note: Homebrew version may not match your Chrome version
# Check compatibility if you encounter issues
```

#### Option 3: Manual Installation
1. Visit: https://chromedriver.chromium.org/downloads
2. Download the version matching your Chrome browser
3. Extract the ZIP file
4. Copy `chromedriver` to `/usr/local/bin/` or add to PATH

### Linux Setup
```bash
# Check Chrome version
google-chrome --version

# Download matching ChromeDriver
wget https://storage.googleapis.com/chrome-for-testing-public/147.0.7727.117/linux64/chromedriver-linux64.zip
unzip chromedriver-linux64.zip
sudo cp chromedriver-linux64/chromedriver /usr/local/bin/

# Make executable
sudo chmod +x /usr/local/bin/chromedriver
```

### Windows Setup
```powershell
# Download from Chrome for Testing
# https://googlechromelabs.github.io/chrome-for-testing/

# Extract and add to PATH
# Or use Chocolatey:
choco install chromedriver
```

## Environment Variables

### Required for Browser Automation
```bash
# Set browser type
export SCOUT_BROWSER=chrome  # or firefox

# Set port (optional, auto-detects if not set)
export SCOUT_PORT=51096

# Enable debug mode to see browser window
export SCOUT_DEBUG=true
```

### Example Usage
```bash
# Run with Firefox (default)
scout my_script.sct

# Run with Chrome
SCOUT_BROWSER=chrome scout my_script.sct

# Debug mode
SCOUT_DEBUG=true SCOUT_BROWSER=chrome scout my_script.sct
```

## Troubleshooting

### Common Issues

#### "WebDriver session not created"
- **Cause**: ChromeDriver version doesn't match Chrome browser
- **Solution**: Update ChromeDriver to match your Chrome version

#### "geckodriver not found"
- **Cause**: GeckoDriver not installed or not in PATH
- **Solution**: Install GeckoDriver and ensure it's in PATH

#### "Port already in use"
- **Cause**: Multiple Scout instances using same port
- **Solution**: Don't set SCOUT_PORT or use different ports

#### "Browser not found"
- **Cause**: Browser not installed or not in standard location
- **Solution**: Install browser or set custom path (if supported)

### Version Compatibility

#### Checking Versions
```bash
# Firefox
firefox --version
geckodriver --version

# Chrome
"/Applications/Google Chrome.app/Contents/MacOS/Google Chrome" --version  # macOS
google-chrome --version  # Linux
chromedriver --version
```

#### Updating Browsers
```bash
# macOS - Chrome updates automatically
# Firefox updates through Firefox or Homebrew

# Linux
sudo apt-get update && sudo apt-get upgrade firefox
```

## Testing Setup

### Basic Test
```scout
// Test script to verify browser setup
goto "https://httpbin.org/html"

title = $"h1" |> textContent()
print("Page title:", title)
```

Run with:
```bash
SCOUT_BROWSER=chrome scout test_browser.sct
```

### Integration Tests
```bash
# Run browser integration tests
cargo test --package scout-interpreter --test browser_integration
```

## Advanced Configuration

### Custom Browser Paths
Some browsers support custom installation paths via environment variables.

### Proxy Configuration
```bash
export SCOUT_PROXY="http://proxy.company.com:8080"
```

### Headless Mode
Scout runs browsers in headless mode by default for CI/testing. Set `SCOUT_DEBUG=true` to see the browser window.

## CI/CD Setup

For automated testing, ensure CI environment has:
1. Browser installed (Firefox preferred for stability)
2. Matching WebDriver installed
3. Proper environment variables set

Example GitHub Actions:
```yaml
- name: Setup Firefox
  run: |
    sudo apt-get update
    sudo apt-get install firefox-geckodriver
```

## Support

If you encounter issues:
1. Check browser and driver versions match
2. Verify PATH includes driver location
3. Test with `SCOUT_DEBUG=true` to see browser behavior
4. Check Scout logs for detailed error messages

For more help, see the main README.md or open an issue on GitHub.