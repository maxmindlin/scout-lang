#[derive(Debug, Default)]
pub struct Program {
    pub stmts: Vec<StmtKind>,
}

#[derive(Debug, PartialEq, Clone)]
pub enum StmtKind {
    Goto(String),
    Scrape,
}

#[derive(Debug, PartialEq, Clone)]
pub enum ExprKind {}
