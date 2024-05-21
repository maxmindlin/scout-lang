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

impl From<Vec<(Identifier, String)>> for HashLiteral {
    fn from(value: Vec<(Identifier, String)>) -> Self {
        let pairs = HashMap::from_iter(value.iter().map(|(i, s)| (i.clone(), s.clone())));
        Self { pairs }
    }
}
