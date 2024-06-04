use std::sync::Arc;

use futures::lock::Mutex;
use rustyline::{error::ReadlineError, Editor};
use scout_interpreter::{env::Env, eval, ScrapeResultsPtr};
use scout_lexer::Lexer;
use scout_parser::{ast::NodeKind, Parser};

const PROMPT: &str = ">> ";

pub async fn run_repl(
    crawler: &fantoccini::Client,
    results: ScrapeResultsPtr,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut rl = Editor::<()>::new();
    let env = Arc::new(Mutex::new(Env::default()));
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
                let lexer = Lexer::new(&line);
                let mut parser = Parser::new(lexer);
                match parser.parse_program() {
                    Ok(prgm) => {
                        let obj = eval(
                            NodeKind::Program(prgm),
                            crawler,
                            env.clone(),
                            results.clone(),
                        )
                        .await;
                        println!("{:?}", obj);
                        //pprint(Arc::into_inner(obj).unwrap());
                    }
                    Err(e) => println!("parser error: {:#?}", e),
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
    Ok(())
}
