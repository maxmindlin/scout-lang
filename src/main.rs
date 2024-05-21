use std::{env, fs};

use repl::run_repl;
use scout_interpreter::{crawler::Crawler, eval};
use scout_lexer::Lexer;
use scout_parser::{ast::NodeKind, Parser};

mod repl;

fn main() {
    let args: Vec<String> = env::args().collect();
    match args.len() {
        1 => run_repl(),
        2 => {
            let filename = &args[1];
            let contents = fs::read_to_string(filename)
                .expect("unable to read file");
            let mut crawler = Crawler::default();
            let lex = Lexer::new(&contents);
            let mut parser = Parser::new(lex);
            match parser.parse_program() {
                Ok(prgm) => {
                    eval(NodeKind::Program(prgm), &mut crawler);
                },
                Err(e) => println!("parser error: {:#?}", e)
            }
        }
        _ => panic!("unsupported number of args"),
    }
}