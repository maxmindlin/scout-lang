use std::collections::HashMap;

use futures::{future::BoxFuture, FutureExt};
use object::Object;
use scout_parser::ast::{ExprKind, Identifier, NodeKind, Program, StmtKind};

use crate::builtin::BuiltinKind;

pub mod builtin;
pub mod object;

pub async fn eval(node: NodeKind, crawler: &fantoccini::Client) -> Object {
    use NodeKind::*;
    match node {
        Program(p) => eval_program(p, crawler).await,
        Stmt(s) => eval_statement(&s, crawler).await,
        Expr(e) => eval_expression(&e, crawler).await,
    }
}

async fn eval_program(prgm: Program, crawler: &fantoccini::Client) -> Object {
    let mut res = Object::Null;
    for stmt in prgm.stmts {
        let val = eval_statement(&stmt, crawler).await;
        match val {
            Object::Error => return val,
            _ => res = val,
        };
    }
    res
}

async fn eval_statement(stmt: &StmtKind, crawler: &fantoccini::Client) -> Object {
    match stmt {
        StmtKind::Goto(url) => {
            crawler.goto(url.as_str()).await.unwrap();
            Object::Null
        }
        StmtKind::Scrape(defs) => {
            let mut res = HashMap::new();
            for (id, def) in &defs.pairs {
                let val = eval_expression(def, crawler).await;
                res.insert(id.clone(), val);
            }
            Object::Map(res)
        }
        StmtKind::Expr(expr) => eval_expression(expr, crawler).await,
    }
}

fn apply_call<'a>(
    ident: &'a Identifier,
    params: &'a [ExprKind],
    crawler: &'a fantoccini::Client,
    prev: Option<Object>,
) -> BoxFuture<'a, Object> {
    async move {
        let mut obj_params = Vec::new();
        for param in params.iter() {
            let expr = eval_expression(param, crawler).await;
            obj_params.push(expr);
        }
        if let Some(obj) = prev {
            obj_params.insert(0, obj);
        }
        match BuiltinKind::is_from(&ident.name) {
            Some(builtin) => builtin.apply(obj_params).await,
            None => Object::Error,
        }
    }
    .boxed()
}

async fn eval_expression(expr: &ExprKind, crawler: &fantoccini::Client) -> Object {
    match expr {
        ExprKind::Select(selector) => {
            match crawler.find(fantoccini::Locator::Css(selector)).await {
                Ok(node) => Object::Node(node),
                Err(_) => Object::Error,
            }
        }
        ExprKind::Str(s) => Object::Str(s.to_owned()),
        ExprKind::Call(ident, params) => apply_call(ident, params, crawler, None).await,
        ExprKind::Chain(exprs) => {
            let mut prev: Option<Object> = None;
            for expr in exprs {
                let eval = match expr {
                    ExprKind::Call(ident, params) => apply_call(ident, params, crawler, prev).await,
                    ExprKind::Select(selector) => {
                        match crawler.find(fantoccini::Locator::Css(selector)).await {
                            Ok(node) => Object::Node(node),
                            Err(_) => Object::Error,
                        }
                    }
                    _ => Object::Error,
                };
                if eval.is_error() {
                    return eval;
                }
                prev = Some(eval);
            }
            prev.unwrap()
        }
        _ => Object::Error,
    }
}
