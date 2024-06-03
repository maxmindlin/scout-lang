use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use env::EnvPointer;
use fantoccini::Locator;
use futures::{future::BoxFuture, FutureExt};
use object::Object;
use scout_parser::ast::{Block, ExprKind, Identifier, NodeKind, Program, StmtKind};

use crate::{builtin::BuiltinKind, env::Env};

pub mod builtin;
pub mod env;
pub mod object;

pub type EvalResult = Result<Arc<Object>, EvalError>;

// TODO add parameters for better debugging.
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum EvalError {
    TypeMismatch,
    InvalidUsage,
    InvalidFnParams,
    NonFunction,
    UnknownIdent,
    UnknownPrefixOp,
    UnknownInfixOp,
    DuplicateDeclare,
}

pub async fn eval(node: NodeKind, crawler: &fantoccini::Client, env: EnvPointer) -> Arc<Object> {
    use NodeKind::*;
    match node {
        Program(p) => eval_program(p, crawler, env.clone()).await,
        Stmt(s) => eval_statement(&s, crawler, env.clone()).await,
        Expr(e) => eval_expression(&e, crawler, env.clone()).await,
    }
}

async fn eval_program(prgm: Program, crawler: &fantoccini::Client, env: EnvPointer) -> Arc<Object> {
    let mut res = Arc::new(Object::Null);
    for stmt in prgm.stmts {
        let val = eval_statement(&stmt, crawler, env.clone()).await;
        match &*val {
            Object::Error => return val,
            _ => res = val.clone(),
        };
    }
    res
}

fn eval_statement<'a>(
    stmt: &'a StmtKind,
    crawler: &'a fantoccini::Client,
    env: EnvPointer,
) -> BoxFuture<'a, Arc<Object>> {
    async move {
        match stmt {
            StmtKind::Goto(url) => {
                crawler.goto(url.as_str()).await.unwrap();
                Arc::new(Object::Null)
            }
            StmtKind::Scrape(defs) => {
                let mut res = HashMap::new();
                for (id, def) in &defs.pairs {
                    let val = eval_expression(def, crawler, env.clone()).await;
                    res.insert(id.clone(), val);
                }
                Arc::new(Object::Map(res))
            }
            StmtKind::Expr(expr) => eval_expression(expr, crawler, env.clone()).await,
            StmtKind::ForLoop(floop) => {
                let items = eval_expression(&floop.iterable, crawler, env.clone()).await;
                match &*items {
                    Object::List(objs) => {
                        for obj in objs {
                            let mut scope = Env::default();
                            scope.add_outer(env.clone());
                            scope.set(&floop.ident, obj.clone());
                            eval_block(&floop.block, crawler, Arc::new(Mutex::new(scope))).await;
                        }

                        Arc::new(Object::Null)
                    }
                    _ => Arc::new(Object::Error),
                }
            }
        }
    }
    .boxed()
}

async fn eval_block(block: &Block, crawler: &fantoccini::Client, env: EnvPointer) -> Arc<Object> {
    let mut res = Arc::new(Object::Null);
    for stmt in &block.stmts {
        let val = eval_statement(stmt, crawler, Arc::clone(&env)).await;
        match &*val {
            Object::Error => return val,
            _ => res = val.clone(),
        }
    }
    res
}

fn apply_call<'a>(
    ident: &'a Identifier,
    params: &'a [ExprKind],
    crawler: &'a fantoccini::Client,
    prev: Option<Arc<Object>>,
    env: EnvPointer,
) -> BoxFuture<'a, Arc<Object>> {
    async move {
        let mut obj_params = Vec::new();
        for param in params.iter() {
            let expr = eval_expression(param, crawler, env.clone()).await;
            obj_params.push(expr);
        }
        if let Some(obj) = prev {
            obj_params.insert(0, obj);
        }
        match BuiltinKind::is_from(&ident.name) {
            Some(builtin) => builtin.apply(obj_params).await,
            None => Arc::new(Object::Error),
        }
    }
    .boxed()
}

fn eval_expression<'a>(
    expr: &'a ExprKind,
    crawler: &'a fantoccini::Client,
    env: EnvPointer,
) -> BoxFuture<'a, Arc<Object>> {
    async move {
        match expr {
            ExprKind::Select(selector) => match crawler.find(Locator::Css(selector)).await {
                Ok(node) => Arc::new(Object::Node(node)),
                Err(_) => Arc::new(Object::Error),
            },
            ExprKind::SelectAll(selector) => match crawler.find_all(Locator::Css(selector)).await {
                Ok(nodes) => Arc::new(Object::List(
                    nodes
                        .iter()
                        .map(|e| Arc::new(Object::Node(e.clone())))
                        .collect(),
                )),
                Err(_) => Arc::new(Object::Error),
            },
            ExprKind::Str(s) => Arc::new(Object::Str(s.to_owned())),
            ExprKind::Call(ident, params) => {
                apply_call(ident, params, crawler, None, env.clone()).await
            }
            ExprKind::Ident(ident) => match env.lock().unwrap().get(ident) {
                Some(obj) => obj.clone(),
                None => Arc::new(Object::Null),
            },
            ExprKind::Chain(exprs) => {
                let mut prev: Option<Arc<Object>> = None;
                for expr in exprs {
                    let eval = match expr {
                        ExprKind::Call(ident, params) => {
                            apply_call(ident, params, crawler, prev, env.clone()).await
                        }
                        _ => eval_expression(expr, crawler, env.clone()).await,
                    };
                    if eval.is_error() {
                        println!("{expr:?}");
                        return eval;
                    }
                    prev = Some(eval);
                }
                prev.unwrap()
            }
            _ => Arc::new(Object::Error),
        }
    }
    .boxed()
}
