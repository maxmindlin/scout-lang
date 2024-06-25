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
    Assign(Identifier, ExprKind),
    Crawl(CrawlLiteral),
    Expr(ExprKind),
    ForLoop(ForLoop),
    Func(FuncDef),
    Goto(ExprKind),
    IfElse(IfElseLiteral),
    Return(Option<ExprKind>),
    Scrape(HashLiteral),
    Screenshot(String),
    TryCatch(Block, Option<Block>),
    Use(ExprKind),
}

#[derive(Debug, PartialEq, Clone)]
pub enum ExprKind {
    // Literals
    Str(String),
    Number(f64),
    Boolean(bool),
    Ident(Identifier),
    List(Vec<ExprKind>),
    Null,

    // Selects
    Select(String, Option<Identifier>),
    SelectAll(String, Option<Identifier>),

    // Rest
    Call(Identifier, Vec<ExprKind>),
    Chain(Vec<ExprKind>),
    Infix(Box<ExprKind>, TokenKind, Box<ExprKind>),
    Prefix(Box<ExprKind>, TokenKind),
}

#[derive(Debug, PartialEq, Clone)]
pub struct CrawlLiteral {
    pub bindings: Option<CrawlBindings>,
    pub filter: Option<ExprKind>,
    pub body: Block,
}

#[derive(Debug, PartialEq, Clone)]
pub struct CrawlBindings {
    pub link: Identifier,
    pub depth: Identifier,
}

impl CrawlLiteral {
    pub fn new(bindings: Option<CrawlBindings>, filter: Option<ExprKind>, body: Block) -> Self {
        Self {
            bindings,
            filter,
            body,
        }
    }
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
pub struct IfElseLiteral {
    pub if_lit: IfLiteral,
    pub elifs: Vec<IfLiteral>,
    pub else_lit: Option<ElseLiteral>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct IfLiteral {
    pub cond: ExprKind,
    pub block: Block,
}

#[derive(Debug, PartialEq, Clone)]
pub struct ElseLiteral {
    pub block: Block,
}

#[derive(Debug, PartialEq, Clone)]
pub struct FuncDef {
    pub ident: Identifier,
    pub params: Vec<FnParam>,
    pub body: Block,
}

impl FuncDef {
    pub fn new(ident: Identifier, params: Vec<FnParam>, body: Block) -> Self {
        Self {
            ident,
            params,
            body,
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct FnParam {
    pub ident: Identifier,
    pub default: Option<ExprKind>,
}

impl FnParam {
    pub fn new(ident: Identifier, default: Option<ExprKind>) -> Self {
        Self { ident, default }
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
