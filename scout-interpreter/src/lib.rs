use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crawler::Crawler;
use object::Object;
use scout_parser::ast::{ExprKind, Identifier, NodeKind, Program, StmtKind};

use crate::builtin::BuiltinKind;

pub mod builtin;
pub mod crawler;
pub mod object;

pub(crate) type CrawlerPointer = Rc<RefCell<Crawler>>;

pub fn eval(node: NodeKind, crawler: CrawlerPointer) -> Object {
    use NodeKind::*;
    match node {
        Program(p) => eval_program(p, crawler),
        Stmt(s) => eval_statement(&s, crawler),
        Expr(e) => eval_expression(&e, crawler),
    }
}

fn eval_program(prgm: Program, crawler: CrawlerPointer) -> Object {
    let mut res = Object::Null;
    for stmt in prgm.stmts {
        let val = eval_statement(&stmt, Rc::clone(&crawler));
        match val {
            Object::Error => return val,
            _ => res = val,
        };
    }
    res
}

fn eval_statement(stmt: &StmtKind, crawler: CrawlerPointer) -> Object {
    match stmt {
        StmtKind::Goto(url) => {
            crawler.borrow_mut().goto(url.as_str()).unwrap();
            Object::Str(crawler.borrow().status().to_string())
        }
        StmtKind::Scrape(defs) => {
            let mut res = HashMap::new();
            for (id, def) in &defs.pairs {
                let val = eval_expression(def, Rc::clone(&crawler));
                res.insert(id.clone(), val);
            }
            Object::Map(res)
        }
        StmtKind::Expr(expr) => eval_expression(expr, Rc::clone(&crawler)),
    }
}

fn apply_call(
    ident: &Identifier,
    params: &[ExprKind],
    crawler: CrawlerPointer,
    prev: Option<Object>,
) -> Object {
    let mut obj_params: Vec<Object> = params
        .iter()
        .map(|e| eval_expression(e, Rc::clone(&crawler)))
        .collect();
    if let Some(obj) = prev {
        obj_params.insert(0, obj);
    }
    match BuiltinKind::is_from(&ident.name) {
        Some(builtin) => builtin.apply(obj_params),
        None => Object::Error,
    }
}

fn eval_expression(expr: &ExprKind, crawler: CrawlerPointer) -> Object {
    match expr {
        ExprKind::Select(selector) => match crawler.borrow_mut().select(selector) {
            Some(node) => Object::Node(node.html()),
            None => Object::Null,
        },
        ExprKind::Str(s) => Object::Str(s.to_owned()),
        ExprKind::Call(ident, params) => apply_call(ident, params, Rc::clone(&crawler), None),
        ExprKind::Chain(exprs) => {
            let mut prev: Option<Object> = None;
            for expr in exprs {
                let eval = match expr {
                    ExprKind::Call(ident, params) => {
                        apply_call(ident, params, Rc::clone(&crawler), prev)
                    }
                    ExprKind::Select(selector) => match crawler.borrow_mut().select(selector) {
                        Some(node) => Object::Node(node.inner_html()),
                        None => Object::Null,
                    },
                    _ => Object::Error,
                };

                if eval == Object::Error {
                    return eval;
                }

                prev = Some(eval);
            }
            prev.unwrap()
        }
        _ => Object::Error,
    }
}
