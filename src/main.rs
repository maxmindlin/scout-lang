use std::{env, fs, process::Command};

use repl::run_repl;
use scout_interpreter::{builder::InterpreterBuilder, env::Env, eval::ScrapeResultsPtr, EnvVars};

mod repl;

async fn run(
    file: Option<String>,
    results: ScrapeResultsPtr,
) -> Result<(), Box<dyn std::error::Error>> {
    let env = Env::default();
    match file {
        None => run_repl(results, env).await,
        Some(f) => {
            let contents = fs::read_to_string(f)?;
            let interpret = InterpreterBuilder::default()
                .with_results(results)
                .build()
                .await?;
            if let Err(e) = interpret.eval(&contents).await {
                println!("Interpeter error: {:?}", e);
            }

            interpret.finalize().await;

            Ok(())
        }
    }
}

#[tokio::main]
async fn main() {
    let env_vars = envy::from_env::<EnvVars>().expect("error loading env config");
    let args: Vec<String> = env::args().collect();

    let child = Command::new("geckodriver")
        .arg("--port")
        .arg(env_vars.port().to_string())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .expect("error spinning up driver process");

    // sleep to allow driver to start
    std::thread::sleep(std::time::Duration::from_millis(50));

    let results = ScrapeResultsPtr::default();
    if let Err(e) = run(args.get(1).cloned(), results.clone()).await {
        println!("Error: {}", e);
    }
    let json_results = results.lock().await.to_json();
    println!("{}", json_results);

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
