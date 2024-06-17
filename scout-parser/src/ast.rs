use std::{collections::HashMap, fmt::Display};

use scout_lexer::TokenKind;

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
    Goto(ExprKind),
    Scrape(HashLiteral),
    Expr(ExprKind),
    ForLoop(ForLoop),
    If(ExprKind, Block),
    Assign(Identifier, ExprKind),
    Screenshot(String),
    Func(FuncDef),
}

#[derive(Debug, PartialEq, Clone)]
pub enum ExprKind {
    Str(String),
    Number(f64),
    Boolean(bool),
    Ident(Identifier),
    Select(String, Option<Identifier>),
    SelectAll(String, Option<Identifier>),
    Call(Identifier, Vec<ExprKind>),
    Chain(Vec<ExprKind>),
    Infix(Box<ExprKind>, TokenKind, Box<ExprKind>),
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

#[derive(Debug, PartialEq, Clone)]
pub struct FuncDef {
    pub ident: Identifier,
    pub args: Vec<Identifier>,
    pub body: Block,
}

impl FuncDef {
    pub fn new(ident: Identifier, args: Vec<Identifier>, body: Block) -> Self {
        Self { ident, args, body }
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

#[derive(Debug, PartialEq, Clone)]
pub struct ForLoop {
    pub ident: Identifier,
    pub iterable: ExprKind,
    pub block: Block,
}

impl ForLoop {
    pub fn new(ident: Identifier, iterable: ExprKind, block: Block) -> Self {
        Self {
            ident,
            iterable,
            block,
        }
    }
}

#[derive(Debug, PartialEq, Clone, Default)]
pub struct Block {
    pub stmts: Vec<StmtKind>,
}

impl Block {
    pub fn new(stmts: Vec<StmtKind>) -> Self {
        Self { stmts }
    }
}
