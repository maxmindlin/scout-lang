use std::{env, fs, process::Command, sync::Arc};

use futures::lock::Mutex;
use repl::run_repl;
use scout_interpreter::{env::Env, eval, ScrapeResultsPtr};
use scout_lexer::Lexer;
use scout_parser::{ast::NodeKind, Parser};
use serde::Deserialize;

mod repl;

#[derive(Deserialize, Debug)]
struct EnvVars {
    #[serde(default)]
    scout_debug: bool,

    #[serde(default = "default_port")]
    scout_port: usize,

    #[serde(default)]
    scout_proxy: Option<String>,
}

fn default_port() -> usize {
    4444
}

async fn run(
    file: Option<String>,
    crawler: &fantoccini::Client,
    results: ScrapeResultsPtr,
) -> Result<(), Box<dyn std::error::Error>> {
    match file {
        None => run_repl(crawler, results).await,
        Some(f) => {
            let contents = fs::read_to_string(f)?;

            let lex = Lexer::new(&contents);
            let mut parser = Parser::new(lex);
            let env = Arc::new(Mutex::new(Env::default()));
            match parser.parse_program() {
                Ok(prgm) => {
                    let res = eval(NodeKind::Program(prgm), crawler, env, results).await;
                    match res {
                        Ok(_) => {}
                        Err(e) => println!("Interpeter error: {:?}", e),
                    };
                    Ok(())
                }
                Err(e) => Err(format!("Parser error: {:#?}", e).into()),
            }
        }
    }
}

#[tokio::main]
async fn main() {
    let env_vars = envy::from_env::<EnvVars>().expect("error loading env config");
    let args: Vec<String> = env::args().collect();

    let child = Command::new("geckodriver")
        .arg("--port")
        .arg(env_vars.scout_port.to_string())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .expect("error spinning up driver process");

    // sleep to allow driver to start
    std::thread::sleep(std::time::Duration::from_millis(50));

    let mut caps = serde_json::map::Map::new();
    if !env_vars.scout_debug {
        let opts = serde_json::json!({ "args": ["--headless"] });
        caps.insert("moz:firefoxOptions".into(), opts);
    }
    if let Some(proxy) = env_vars.scout_proxy {
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
        .expect("error starting browser");

    let results = ScrapeResultsPtr::default();
    if let Err(e) = run(args.get(1).cloned(), &crawler, results.clone()).await {
        println!("Error: {}", e);
    }
    let json_results = results.lock().await.to_json();
    println!("{}", json_results);
    let _ = crawler.close().await;

    #[cfg(target_os = "windows")]
    let mut kill = Command::new("taskkill")
        .arg("/PID")
        .arg(&child.id().to_string())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .arg("/F")
        .spawn()
        .expect("error sending driver kill");

    #[cfg(not(target_os = "windows"))]
    let mut kill = Command::new("kill")
        .args(["-s", "TERM", &child.id().to_string()])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .expect("error sending driver kill");

    kill.wait().expect("error waiting for driver kill");
}
