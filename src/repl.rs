use rustyline::{error::ReadlineError, Editor};
use scout_interpreter::{crawler::Crawler, eval};
use scout_lexer::Lexer;
use scout_parser::{ast::NodeKind, Parser};

const PROMPT: &str = ">> ";

pub fn run_repl() {
    let mut rl = Editor::<()>::new();
    let mut crawler = Crawler::default();
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
                        eval(NodeKind::Program(prgm), &mut crawler);
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
}