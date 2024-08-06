use std::{env, fs};

use repl::run_repl;
use scout_interpreter::{builder::InterpreterBuilder, Interpreter};

mod repl;

async fn run(
    file: Option<String>,
    interpreter: &Interpreter,
) -> Result<(), Box<dyn std::error::Error>> {
    match file {
        None => run_repl(interpreter).await,
        Some(f) => {
            let contents = fs::read_to_string(f)?;
            if let Err(e) = interpreter.eval(&contents).await {
                println!("Interpeter error: {:?}", e);
            }

            Ok(())
        }
    }
}

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();

    let interpreter = InterpreterBuilder::default()
        .build()
        .await
        .expect("failed to build interpreter");
    if let Err(e) = run(args.get(1).cloned(), &interpreter).await {
        println!("Error: {}", e);
    }
    let json_results = interpreter.results().lock().await.to_json();
    println!("{}", json_results);

    interpreter.close().await;
}
