use std::sync::Arc;

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

pub struct Interpreter {
    env: EnvPointer,
    results: ScrapeResultsPtr,
    crawler: fantoccini::Client,
}

impl Interpreter {
    pub fn new(env: EnvPointer, results: ScrapeResultsPtr, crawler: fantoccini::Client) -> Self {
        Self {
            env,
            results,
            crawler,
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

    pub async fn finalize(self) {
        let _ = self.crawler.close().await;
    }
}

impl From<EvalError> for InterpreterError {
    fn from(value: EvalError) -> Self {
        InterpreterError::EvalError(value)
    }
}
