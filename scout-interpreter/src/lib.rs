use std::{
    process::{Child, Command},
    sync::Arc,
};

use env::EnvPointer;
use eval::{eval, EvalError, ScrapeResultsPtr};
use object::Object;
use scout_lexer::Lexer;
use scout_parser::{ast::NodeKind, ParseError, Parser};
use serde::Deserialize;

pub mod builder;
pub mod builtin;
pub mod env;
pub mod eval;
pub mod import;
pub mod object;

#[derive(Deserialize, Debug)]
pub struct EnvVars {
    #[serde(default)]
    scout_debug: bool,

    #[serde(default = "default_port")]
    scout_port: usize,

    #[serde(default)]
    scout_proxy: Option<String>,
}

impl EnvVars {
    pub fn debug(&self) -> bool {
        self.scout_debug
    }

    pub fn port(&self) -> usize {
        self.scout_port
    }

    pub fn proxy(&self) -> &Option<String> {
        &self.scout_proxy
    }
}

fn default_port() -> usize {
    4444
}

#[derive(Debug)]
pub enum InterpreterError {
    EvalError(EvalError),
    ParserError(ParseError),
}

pub struct GeckDriverProc(Child);

impl GeckDriverProc {
    pub fn new(port: usize) -> Self {
        let child = Command::new("geckodriver")
            .arg("--port")
            .arg(port.to_string())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn()
            .expect("error spinning up driver process");

        // sleep to allow driver to start
        std::thread::sleep(std::time::Duration::from_millis(50));
        Self(child)
    }
}

pub struct Interpreter {
    env: EnvPointer,
    results: ScrapeResultsPtr,
    crawler: fantoccini::Client,
    _geckodriver_proc: GeckDriverProc,
}

impl Interpreter {
    pub fn new(
        env: EnvPointer,
        results: ScrapeResultsPtr,
        crawler: fantoccini::Client,
        geckodriver_proc: GeckDriverProc,
    ) -> Self {
        Self {
            env,
            results,
            crawler,
            _geckodriver_proc: geckodriver_proc,
        }
    }
    pub async fn eval(&self, content: &str) -> Result<Arc<Object>, InterpreterError> {
        let lexer = Lexer::new(content);
        let mut parser = Parser::new(lexer);
        match parser.parse_program() {
            Ok(prgm) => Ok(eval(
                NodeKind::Program(prgm),
                &self.crawler,
                self.env.clone(),
                self.results.clone(),
            )
            .await?),
            Err(e) => Err(InterpreterError::ParserError(e)),
        }
    }

    pub fn results(&self) -> ScrapeResultsPtr {
        self.results.clone()
    }

    pub fn reset(&mut self) {
        self.env = EnvPointer::default();
        self.results = ScrapeResultsPtr::default();
    }

    pub async fn close(self) {
        let _ = self.crawler.close().await;
    }
}

impl From<EvalError> for InterpreterError {
    fn from(value: EvalError) -> Self {
        InterpreterError::EvalError(value)
    }
}

impl Drop for GeckDriverProc {
    fn drop(&mut self) {
        #[cfg(target_os = "windows")]
        let mut kill = Command::new("taskkill")
            .arg("/PID")
            .arg(&self.0.id().to_string())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .arg("/F")
            .spawn()
            .expect("error sending driver kill");

        #[cfg(not(target_os = "windows"))]
        let mut kill = Command::new("kill")
            .args(["-s", "TERM", &self.0.id().to_string()])
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn()
            .expect("error sending driver kill");

        kill.wait().expect("error waiting for driver kill");
    }
}
