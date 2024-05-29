use std::{cell::RefCell, rc::Rc};

use rustyline::{error::ReadlineError, Editor};
use scout_interpreter::{crawler::Crawler, eval};
use scout_lexer::Lexer;
use scout_parser::{ast::NodeKind, Parser};

use crate::print::pprint;

const PROMPT: &str = ">> ";

pub fn run_repl() {
    let mut rl = Editor::<()>::new();
    let crawler = Crawler::default();
    let crwl_pt = Rc::new(RefCell::new(crawler));
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
                        let obj = eval(NodeKind::Program(prgm), Rc::clone(&crwl_pt));
                        pprint(obj);
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
