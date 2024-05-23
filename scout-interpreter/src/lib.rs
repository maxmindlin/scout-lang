use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crawler::Crawler;
use object::Object;
use scout_parser::ast::{ExprKind, NodeKind, Program, StmtKind};

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
            Object::Null
        }
        StmtKind::Scrape(defs) => {
            let mut res = HashMap::new();
            for (id, def) in &defs.pairs {
                let val = eval_expression(def, Rc::clone(&crawler));
                res.insert(id.clone(), val);
            }
            Object::Map(res)
        }
    }
}

fn eval_expression(expr: &ExprKind, crawler: CrawlerPointer) -> Object {
    match expr {
        ExprKind::Select(selector) => match crawler.borrow_mut().select(selector) {
            Some(node) => Object::Node(node.inner_html()),
            None => Object::Null,
        },
        _ => Object::Error,
    }
}
