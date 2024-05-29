use std::{cell::RefCell, env, fs, rc::Rc};

use print::pprint;
use repl::run_repl;
use scout_interpreter::{crawler::Crawler, eval};
use scout_lexer::Lexer;
use scout_parser::{ast::NodeKind, Parser};

mod print;
mod repl;

fn main() {
    let args: Vec<String> = env::args().collect();
    match args.len() {
        1 => run_repl(),
        2 => {
            let filename = &args[1];
            let contents = fs::read_to_string(filename).expect("unable to read file");
            let crawler = Crawler::default();
            let crwl_pt = Rc::new(RefCell::new(crawler));
            let lex = Lexer::new(&contents);
            let mut parser = Parser::new(lex);
            match parser.parse_program() {
                Ok(prgm) => {
                    let res = eval(NodeKind::Program(prgm), crwl_pt);
                    pprint(res);
                }
                Err(e) => println!("parser error: {:#?}", e),
            }
        }
        _ => panic!("unsupported number of args"),
    }
}
