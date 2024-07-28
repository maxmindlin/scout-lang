use std::sync::Arc;

use futures::lock::Mutex;
use rustyline::{error::ReadlineError, Editor};
use scout_interpreter::{
    builder::InterpreterBuilder, env::Env, eval::ScrapeResultsPtr, Interpreter,
};

const PROMPT: &str = ">> ";

pub async fn run_repl(
    // results: ScrapeResultsPtr,
    interpreter: &Interpreter, // env: Env,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut rl = Editor::<()>::new();
    // let env = Arc::new(Mutex::new(env));

    // let interpreter = InterpreterBuilder::default()
    //     .with_env(env.clone())
    //     .with_results(results.clone())
    //     .build()
    //     .await?;
    if rl.load_history("history.txt").is_err() {
        println!("No previous history.");
    }
    println!("Welcome to the ScoutLang programming language");
    println!("Press CTRL-c to exit.");
    loop {
        let readline = rl.readline(PROMPT);
        match readline {
            Ok(line) => {
                rl.add_history_entry(line.as_str());

                match interpreter.eval(&line).await {
                    Ok(o) => println!("{}", o.to_display().await),
                    Err(e) => println!("Interpeter error: {:?}", e),
                }
            }
            Err(ReadlineError::Interrupted) => {
                println!("Exiting.");
                break;
            }
            Err(ReadlineError::Eof) => {
                println!("Eof");
                break;
            }
            Err(err) => {
                println!("Error: {:?}", err);
                break;
            }
        }
    }
    rl.save_history("history.txt").unwrap();
    // interpreter.close().await;
    Ok(())
}
