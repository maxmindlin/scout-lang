use std::collections::HashMap;

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

#[derive(Debug, PartialEq, Clone, Eq)]
pub enum ExprKind {
    Select(String),
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

#[derive(Default, Debug, PartialEq, Eq, Clone)]
pub struct HashLiteral {
    pub pairs: HashMap<Identifier, ExprKind>,
}

impl From<Vec<(Identifier, ExprKind)>> for HashLiteral {
    fn from(value: Vec<(Identifier, ExprKind)>) -> Self {
        let pairs = HashMap::from_iter(value.iter().map(|(i, s)| (i.clone(), s.clone())));
        Self { pairs }
    }
}
