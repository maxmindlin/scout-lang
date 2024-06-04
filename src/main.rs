use std::{
    env, fs,
    process::Command,
    sync::{Arc, Mutex},
};

use repl::run_repl;
use scout_interpreter::{env::Env, eval};
use scout_lexer::Lexer;
use scout_parser::{ast::NodeKind, Parser};

mod repl;

async fn run(
    args: Vec<String>,
    crawler: &fantoccini::Client,
) -> Result<(), Box<dyn std::error::Error>> {
    match args.len() {
        1 => run_repl(crawler).await,
        2 => {
            let filename = &args[1];
            let contents = fs::read_to_string(filename)?;

            let lex = Lexer::new(&contents);
            let mut parser = Parser::new(lex);
            let env = Arc::new(Mutex::new(Env::default()));
            match parser.parse_program() {
                Ok(prgm) => {
                    let res = eval(NodeKind::Program(prgm), crawler, env).await;
                    println!("{:?}", res);
                    Ok(())
                    // pprint(Arc::into_inner(res).unwrap());
                }
                Err(e) => Err(format!("parser error: {:#?}", e).into()),
            }
        }
        _ => Err("unsupported number of args".into()),
    }
}

#[tokio::main]
async fn main() {
    let child = Command::new("geckodriver")
        .arg("--port")
        .arg("4444")
        .spawn()
        .expect("error spinning up driver process");
    let crawler = fantoccini::ClientBuilder::native()
        .connect("http://localhost:4444")
        .await
        .expect("error starting browser");
    let args: Vec<String> = env::args().collect();
    if let Err(e) = run(args, &crawler).await {
        println!("Error: {}", e);
    }
    let _ = crawler.close().await;

    #[cfg(target_os = "windows")]
    let mut kill = Command::new("taskkill")
        .arg("/PID")
        .arg(&child.id().to_string())
        .arg("/F")
        .spawn()
        .expect("error sending driver kill");

    #[cfg(not(target_os = "windows"))]
    let mut kill = Command::new("kill")
        .args(["-s", "TERM", &child.id().to_string()])
        .spawn()
        .expect("error sending driver kill");

    kill.wait().expect("error waiting for driver kill");
}
