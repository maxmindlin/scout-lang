<div align="center">
<img src="./assets/scout-logo.png" width="400">
<p style="font-size:0.5em;color:#d4d4d4">A Web Crawling Programming Language</p>
<img alt="GitHub Actions Workflow Status" src="https://img.shields.io/badge/license-MIT%2FApache-blue.svg?style=for-the-badge&label=License">
<img alt="GitHub Release" src="https://img.shields.io/github/v/release/maxmindlin/scout-lang?style=for-the-badge">
<img alt="GitHub Actions Workflow Status" src="https://img.shields.io/github/actions/workflow/status/maxmindlin/scout-lang/ci.yml?style=for-the-badge&label=CI">
</div>
<hr>
<br>

ScoutLang is a DSL made for web scraping, focusing on a simple and expressive syntax. A powerful web crawling stack is abstracted away, allowing you to write powerful, easy to read scraping scripts.

## Why Scout?

- Gain access to powerful web scraping technology without needing expertise
- A focus on developer velocity
- Builtin debugging tools

<br>

![example](./assets/code-sample.png)

## Iterative script building

ScoutLang comes bundled with a full REPL and a powerful debugging mode, allowing you to visualize your web scraping scripts in real time. 

![debug](./assets/scout.gif)

# Installation

Eventually Scout installation will come bundled with the necessary pre-reqs. For now, you will need:
- Some version of FireFox
- [Geckodriver](https://github.com/mozilla/geckodriver)

Run the installer:

```bash
curl --proto '=https' --tlsv1.2 -LsSf https://raw.githubusercontent.com/maxmindlin/scout-lang/main/scripts/installer.py | python3
```

# Usage

The `scout` binary ran with a filename will read and interpret a script file. Without a script will start the REPL.

Available ENV variables:
- `SCOUT_DEBUG`: Whether or not to open the debug browser. Defaults to `false`.
- `SCOUT_PORT`: Which port to run Scout on. Defaults to `4444`.
- `SCOUT_PROXY`: An optional URL to proxy requests to. Defaults to none.
- `SCOUT_PATH`: A path to where Scout installs dependencies, like the standard lib. Defaults to `$HOME/scout-lang/`.

# License

Scout is dual-licensed with MIT & Apache 2.0, at your option.
