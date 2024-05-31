use std::{env, fs};

use print::pprint;
use repl::run_repl;
use scout_interpreter::eval;
use scout_lexer::Lexer;
use scout_parser::{ast::NodeKind, Parser};

mod print;
mod repl;

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();
    match args.len() {
        1 => run_repl().await,
        2 => {
            let filename = &args[1];
            let contents = fs::read_to_string(filename).expect("unable to read file");
            let crawler = fantoccini::ClientBuilder::native()
                .connect("http://localhost:4444")
                .await
                .expect("error creating driver");
            let lex = Lexer::new(&contents);
            let mut parser = Parser::new(lex);
            match parser.parse_program() {
                Ok(prgm) => {
                    let res = eval(NodeKind::Program(prgm), &crawler).await;
                    pprint(res);
                }
                Err(e) => println!("parser error: {:#?}", e),
            }
        }
        _ => panic!("unsupported number of args"),
    }
}
