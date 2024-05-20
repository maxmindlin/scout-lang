use std::collections::HashMap;

#[derive(Debug, Default)]
pub struct Program {
    pub stmts: Vec<StmtKind>,
}

#[derive(Debug, PartialEq, Clone)]
pub enum StmtKind {
    Goto(String),
    Scrape(HashLiteral),
}

#[derive(Debug, PartialEq, Clone, Eq)]
pub enum ExprKind {}

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
pub struct HashLiteral {
    pub pairs: HashMap<Identifier, String>,
}