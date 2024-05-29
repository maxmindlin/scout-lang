use std::{collections::HashMap, fmt::Display};

#[derive(Debug)]
pub enum NodeKind {
    Program(Program),
    Stmt(StmtKind),
    Expr(ExprKind),
}

#[derive(Debug, Default)]
pub struct Program {
    pub stmts: Vec<StmtKind>,
}

#[derive(Debug, PartialEq, Clone)]
pub enum StmtKind {
    Goto(String),
    Scrape(HashLiteral),
}

#[derive(Debug, PartialEq, Clone)]
pub enum ExprKind {
    Str(String),
    Number(f64),
    Boolean(bool),
    Select(String),
    Call(Identifier, Vec<ExprKind>),
    Chain(Vec<ExprKind>),
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct Identifier {
    pub name: String,
}

impl Identifier {
    pub fn new(name: String) -> Self {
        Self { name }
    }
}

#[derive(Default, Debug, PartialEq, Eq, Clone)]
pub struct SelectLiteral {
    pub selector: String,
}

impl SelectLiteral {
    pub fn new(selector: String) -> Self {
        Self { selector }
    }
}

#[derive(Default, Debug, PartialEq, Clone)]
pub struct HashLiteral {
    pub pairs: HashMap<Identifier, ExprKind>,
}

impl From<Vec<(Identifier, ExprKind)>> for HashLiteral {
    fn from(value: Vec<(Identifier, ExprKind)>) -> Self {
        let pairs = HashMap::from_iter(value.iter().map(|(i, s)| (i.clone(), s.clone())));
        Self { pairs }
    }
}

impl Display for Identifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}
