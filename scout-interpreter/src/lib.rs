use std::thread::sleep;
use std::time::Duration;
use std::{collections::HashMap, sync::Arc};

use env::EnvPointer;
use fantoccini::Locator;
use futures::lock::Mutex;
use futures::{future::BoxFuture, FutureExt};
use object::{obj_map_to_json, Object};
use scout_lexer::TokenKind;
use scout_parser::ast::{Block, ExprKind, Identifier, NodeKind, Program, StmtKind};
use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};

use crate::{builtin::BuiltinKind, env::Env};

pub mod builtin;
pub mod env;
pub mod object;

pub type EvalResult = Result<Arc<Object>, EvalError>;
pub type ScrapeResultsPtr = Arc<Mutex<ScrapeResults>>;

#[derive(Default, Serialize, Deserialize, Debug)]
pub struct ScrapeResults {
    results: Map<String, Value>,
}

impl ScrapeResults {
    pub fn add_result(&mut self, res: Map<String, Value>, url: &str) {
        match self.results.get_mut(url) {
            None => {
                self.results.insert(url.to_owned(), vec![res].into());
            }
            Some(Value::Array(v)) => {
                v.push(Value::from(res));
            }
            // This should never happen since `add_results` is the only way to
            // insert to the map.
            _ => panic!("results was not a vec type"),
        }
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(self).unwrap()
    }
}

// TODO add parameters for better debugging.
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum EvalError {
    TypeMismatch,
    InvalidUsage,
    InvalidFnParams,
    InvalidExpr,
    InvalidUrl,
    NonFunction,
    UnknownIdent,
    UnknownPrefixOp,
    UnknownInfixOp,
    DuplicateDeclare,
    NonIterable,
    ScreenshotError,
    BrowserError,
}

pub async fn eval(
    node: NodeKind,
    crawler: &fantoccini::Client,
    env: EnvPointer,
    results: ScrapeResultsPtr,
) -> EvalResult {
    use NodeKind::*;
    match node {
        Program(p) => eval_program(p, crawler, env.clone(), results.clone()).await,
        Stmt(s) => eval_statement(&s, crawler, env.clone(), results.clone()).await,
        Expr(e) => eval_expression(&e, crawler, env.clone(), results.clone()).await,
    }
}

async fn eval_program(
    prgm: Program,
    crawler: &fantoccini::Client,
    env: EnvPointer,
    results: ScrapeResultsPtr,
) -> EvalResult {
    let mut res = Arc::new(Object::Null);
    for stmt in prgm.stmts {
        res = eval_statement(&stmt, crawler, env.clone(), results.clone())
            .await?
            .clone();
    }
    Ok(res)
}

fn eval_statement<'a>(
    stmt: &'a StmtKind,
    crawler: &'a fantoccini::Client,
    env: EnvPointer,
    results: ScrapeResultsPtr,
) -> BoxFuture<'a, EvalResult> {
    async move {
        match stmt {
            StmtKind::Goto(expr) => {
                if let Object::Str(url) =
                    &*eval_expression(expr, crawler, env.clone(), results.clone()).await?
                {
                    if crawler.goto(url.as_str()).await.is_err() {
                        return Err(EvalError::InvalidUrl);
                    };
                } else {
                    return Err(EvalError::InvalidFnParams);
                }

                // @TODO: Need a better way to determine that a page is "done"
                sleep(Duration::from_secs(1));

                Ok(Arc::new(Object::Null))
            }
            StmtKind::Scrape(defs) => {
                let mut res = HashMap::new();
                for (id, def) in &defs.pairs {
                    let val = eval_expression(def, crawler, env.clone(), results.clone()).await?;
                    res.insert(id.clone(), val);
                }
                results.lock().await.add_result(
                    obj_map_to_json(&res),
                    crawler.current_url().await.unwrap().as_str(),
                );
                Ok(Arc::new(Object::Null))
            }
            StmtKind::Expr(expr) => {
                eval_expression(expr, crawler, env.clone(), results.clone()).await
            }
            StmtKind::ForLoop(floop) => {
                let items =
                    eval_expression(&floop.iterable, crawler, env.clone(), results.clone()).await?;
                match &*items {
                    Object::List(objs) => {
                        for obj in objs {
                            let mut scope = Env::default();
                            scope.add_outer(env.clone()).await;
                            scope.set(&floop.ident, obj.clone()).await;
                            eval_block(
                                &floop.block,
                                crawler,
                                Arc::new(Mutex::new(scope)),
                                results.clone(),
                            )
                            .await?;
                        }

                        Ok(Arc::new(Object::Null))
                    }
                    _ => Err(EvalError::NonIterable),
                }
            }
            StmtKind::Assign(ident, expr) => {
                let val = eval_expression(expr, crawler, env.clone(), results.clone()).await?;
                env.lock().await.set(ident, val).await;
                Ok(Arc::new(Object::Null))
            }
            StmtKind::Screenshot(path) => {
                let png = crawler.screenshot().await?;
                let img = image::io::Reader::new(std::io::Cursor::new(png))
                    .with_guessed_format()
                    .map_err(|_| EvalError::ScreenshotError)?
                    .decode()?;
                img.save(path)?;

                Ok(Arc::new(Object::Null))
            }
            StmtKind::If(cond, block) => {
                let truth_check =
                    eval_expression(cond, crawler, env.clone(), results.clone()).await?;
                if truth_check.is_truthy() {
                    eval_block(block, crawler, env.clone(), results.clone()).await?;
                }

                Ok(Arc::new(Object::Null))
            }
        }
    }
    .boxed()
}

async fn eval_block(
    block: &Block,
    crawler: &fantoccini::Client,
    env: EnvPointer,
    results: ScrapeResultsPtr,
) -> EvalResult {
    let mut res = Arc::new(Object::Null);
    for stmt in &block.stmts {
        res = eval_statement(stmt, crawler, env.clone(), results.clone())
            .await?
            .clone();
    }
    Ok(res)
}

fn apply_call<'a>(
    ident: &'a Identifier,
    params: &'a [ExprKind],
    crawler: &'a fantoccini::Client,
    prev: Option<Arc<Object>>,
    env: EnvPointer,
    results: ScrapeResultsPtr,
) -> BoxFuture<'a, EvalResult> {
    async move {
        let mut obj_params = Vec::new();
        for param in params.iter() {
            let expr = eval_expression(param, crawler, env.clone(), results.clone()).await?;
            obj_params.push(expr);
        }
        if let Some(obj) = prev {
            obj_params.insert(0, obj);
        }
        match BuiltinKind::is_from(&ident.name) {
            Some(builtin) => builtin.apply(crawler, results.clone(), obj_params).await,
            None => Err(EvalError::UnknownIdent),
        }
    }
    .boxed()
}

async fn apply_debug_border(crawler: &fantoccini::Client, selector: &str) {
    let js = r#"
    const [selector] = arguments;

    document.querySelector(selector).style.boxShadow = "0 0 0 5px red";
    document.querySelector(selector).style.outline = "dashed 5px yellow";
    "#;
    let _ = crawler.execute(js, vec![json!(selector)]).await;
}

async fn apply_debug_border_all(crawler: &fantoccini::Client, selector: &str) {
    let js = r#"
    const [selector] = arguments;

    document.querySelectorAll(selector).forEach(elem => elem.style.boxShadow = "0 0 0 5px red");
    document.querySelectorAll(selector).forEach(elem => elem.style.outline = "dashed 5px yellow");
    "#;
    let _ = crawler.execute(js, vec![json!(selector)]).await;
}

fn eval_expression<'a>(
    expr: &'a ExprKind,
    crawler: &'a fantoccini::Client,
    env: EnvPointer,
    results: ScrapeResultsPtr,
) -> BoxFuture<'a, EvalResult> {
    async move {
        match expr {
            ExprKind::Select(selector, scope) => match scope {
                Some(ident) => match env.lock().await.get(ident).await.as_deref() {
                    Some(Object::Node(elem)) => match elem.find(Locator::Css(selector)).await {
                        Ok(node) => {
                            // @TODO fix - applies borders outside scope
                            apply_debug_border(crawler, selector).await;
                            Ok(Arc::new(Object::Node(node)))
                        }
                        Err(_) => Ok(Arc::new(Object::Null)),
                    },
                    Some(_) => Err(EvalError::InvalidUsage),
                    None => Err(EvalError::UnknownIdent),
                },
                None => match crawler.find(Locator::Css(selector)).await {
                    Ok(node) => {
                        apply_debug_border(crawler, selector).await;
                        Ok(Arc::new(Object::Node(node)))
                    }
                    Err(_) => Ok(Arc::new(Object::Null)),
                },
            },
            ExprKind::SelectAll(selector, scope) => match scope {
                Some(ident) => match env.lock().await.get(ident).await.as_deref() {
                    Some(Object::Node(elem)) => match elem.find_all(Locator::Css(selector)).await {
                        Ok(nodes) => {
                            // @TODO fix - applies borders outside scope
                            apply_debug_border_all(crawler, selector).await;
                            let elems = nodes
                                .iter()
                                .map(|e| Arc::new(Object::Node(e.clone())))
                                .collect();
                            Ok(Arc::new(Object::List(elems)))
                        }
                        Err(_) => Ok(Arc::new(Object::Null)),
                    },
                    Some(_) => Err(EvalError::InvalidUsage),
                    None => Err(EvalError::UnknownIdent),
                },
                None => match crawler.find_all(Locator::Css(selector)).await {
                    Ok(nodes) => {
                        apply_debug_border_all(crawler, selector).await;
                        let elems = nodes
                            .iter()
                            .map(|e| Arc::new(Object::Node(e.clone())))
                            .collect();
                        Ok(Arc::new(Object::List(elems)))
                    }
                    Err(_) => Ok(Arc::new(Object::Null)),
                },
            },
            ExprKind::Str(s) => Ok(Arc::new(Object::Str(s.to_owned()))),
            ExprKind::Number(n) => Ok(Arc::new(Object::Number(*n))),
            ExprKind::Call(ident, params) => {
                apply_call(ident, params, crawler, None, env.clone(), results.clone()).await
            }
            ExprKind::Ident(ident) => match env.lock().await.get(ident).await {
                Some(obj) => Ok(obj.clone()),
                None => Err(EvalError::UnknownIdent),
            },
            ExprKind::Chain(exprs) => {
                let mut prev: Option<Arc<Object>> = None;
                for expr in exprs {
                    let eval = match expr {
                        ExprKind::Call(ident, params) => {
                            apply_call(ident, params, crawler, prev, env.clone(), results.clone())
                                .await?
                        }
                        _ => eval_expression(expr, crawler, env.clone(), results.clone()).await?,
                    };
                    prev = Some(eval);
                }
                Ok(prev.unwrap())
            }
            ExprKind::Infix(lhs, op, rhs) => {
                // TODO: precedence....
                let l_obj = eval_expression(lhs, crawler, env.clone(), results.clone()).await?;
                let r_obj = eval_expression(rhs, crawler, env.clone(), results.clone()).await?;
                let res = eval_op(l_obj.clone(), op, r_obj.clone())?;
                Ok(res)
            }
            ExprKind::Boolean(val) => Ok(Arc::new(Object::Boolean(*val))),
        }
    }
    .boxed()
}

fn eval_op(lhs: Arc<Object>, op: &TokenKind, rhs: Arc<Object>) -> EvalResult {
    match (lhs.clone(), op, rhs.clone()) {
        (_, TokenKind::EQ, _) => Ok(Arc::new(Object::Boolean(lhs == rhs))),
        (_, TokenKind::NEQ, _) => Ok(Arc::new(Object::Boolean(lhs != rhs))),
        _ => Err(EvalError::UnknownInfixOp),
    }
}

impl From<fantoccini::error::CmdError> for EvalError {
    fn from(_: fantoccini::error::CmdError) -> Self {
        Self::BrowserError
    }
}

impl From<image::ImageError> for EvalError {
    fn from(_: image::ImageError) -> Self {
        Self::ScreenshotError
    }
}
