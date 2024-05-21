use std::collections::HashMap;

use crawler::Crawler;
use object::Object;
use scout_parser::ast::{ExprKind, HashLiteral, NodeKind, Program, StmtKind};

pub mod builtin;
pub mod crawler;
pub mod object;

pub fn eval(node: NodeKind, crawler: &mut Crawler) -> Object {
    use NodeKind::*;
    match node {
        Program(p) => eval_program(p, crawler),
        Stmt(s) => eval_statement(&s, crawler),
        Expr(e) => eval_expression(&e, crawler),
    }
}

fn eval_program(prgm: Program, crawler: &mut Crawler) -> Object {
    let mut res = Object::Null;
    for stmt in prgm.stmts {
        let val = eval_statement(&stmt, crawler);
        match val {
            Object::Error => return val,
            _ => res = val,
        };
    }
    res
}

fn eval_statement(stmt: &StmtKind, crawler: &mut Crawler) -> Object {
    match stmt {
        StmtKind::Goto(url) => {
            crawler.goto(url.as_str()).unwrap();
            Object::Null
        }
        StmtKind::Scrape(defs) => {
            let mut res = HashMap::new();
            for (id, def) in &defs.pairs {
                let scrape_res = crawler.scrape(&def);
                res.insert(id.clone(), scrape_res);
            }
            let lit = HashLiteral { pairs: res };
            Object::Map(lit)
        }
    }
}

fn eval_expression(expr: &ExprKind, crawler: &mut Crawler) -> Object {
    Object::Null
}