use std::{collections::HashMap, fmt::Display};

use scout_lexer::Token;

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
    Assign(ExprKind, ExprKind, bool),
    Crawl(CrawlLiteral),
    Expr(ExprKind),
    ForLoop(ForLoop),
    WhileLoop(ExprKind, Block),
    Func(FuncDef, bool),
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
    Map(HashLiteral),
    Null,

    // Selects
    Select(String, Option<Identifier>),
    SelectAll(String, Option<Identifier>),

    // Rest
    Call(CallLiteral),
    Chain(Vec<ExprKind>),
    Infix(Box<ExprKind>, Token, Box<ExprKind>),
    Prefix(Box<ExprKind>, Token),
}

#[derive(Debug, PartialEq, Clone)]
pub struct CallLiteral {
    pub ident: Identifier,
    pub args: Vec<ExprKind>,
    pub kwargs: Vec<Kwarg>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Kwarg {
    pub ident: Identifier,
    pub expr: ExprKind,
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

impl std::fmt::Display for FnParam {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.ident)?;
        if let Some(default) = &self.default {
            write!(f, " = {default}")?;
        }

        Ok(())
    }
}

impl std::fmt::Display for FuncDef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut param_str = String::new();
        for (idx, param) in self.params.iter().enumerate() {
            param_str.push_str(param.to_string().as_str());
            if idx != self.params.len() - 1 {
                param_str.push_str(", ");
            }
        }
        writeln!(f, "def {}({param_str}) do\n{}\nend", self.ident, self.body)
    }
}

impl std::fmt::Display for CallLiteral {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}(", self.ident)?;
        for (idx, arg) in self.args.iter().enumerate() {
            write!(f, "{arg}")?;
            if idx != self.args.len() - 1 {
                write!(f, ", ")?;
            }
        }

        if !self.kwargs.is_empty() {
            write!(f, ", ")?;

            for (idx, kwarg) in self.kwargs.iter().enumerate() {
                write!(f, "{kwarg}")?;
                if idx != self.kwargs.len() - 1 {
                    write!(f, ", ")?;
                }
            }
        }

        write!(f, ")")
    }
}

impl std::fmt::Display for Kwarg {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} = {}", self.ident, self.expr)
    }
}

impl std::fmt::Display for HashLiteral {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{{ ")?;
        for (idx, (i, o)) in self.pairs.iter().enumerate() {
            write!(f, "{}: {}", i, o)?;
            if idx != self.pairs.len() - 1 {
                write!(f, ", ")?;
            }
        }
        write!(f, " }}")
    }
}

impl std::fmt::Display for Block {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for stmt in &self.stmts {
            write!(f, "{stmt}\n")?;
        }
        Ok(())
    }
}

impl std::fmt::Display for ExprKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use ExprKind::*;
        match self {
            Str(s) => write!(f, r#""{s}""#),
            Number(n) => write!(f, "{n}"),
            Boolean(b) => write!(f, "{b}"),
            Ident(ident) => write!(f, "{ident}"),
            List(l) => {
                write!(f, "[")?;
                for (i, obj) in l.iter().enumerate() {
                    write!(f, "{obj}")?;
                    if i != l.len() - 1 {
                        write!(f, ", ")?;
                    }
                }

                write!(f, "]")
            }
            Map(hash) => write!(f, "{hash}"),
            Null => write!(f, "null"),
            Select(s, mb_ident) => match mb_ident {
                Some(ident) => write!(f, r#"$({ident})"{s}""#),
                None => write!(f, r#"$"{s}""#),
            },
            SelectAll(s, mb_ident) => match mb_ident {
                Some(ident) => write!(f, r#"$$({ident})"{s}""#),
                None => write!(f, r#"$$"{s}""#),
            },
            Call(lit) => write!(f, "{lit}"),
            Chain(exprs) => {
                for (i, expr) in exprs.iter().enumerate() {
                    write!(f, "{expr}")?;
                    if i != exprs.len() - 1 {
                        write!(f, " |> ")?;
                    }
                }
                Ok(())
            }
            Infix(lhs, op, rhs) => write!(f, "{lhs} {} {rhs}", op.literal),
            Prefix(lhs, op) => write!(f, "{lhs} {}", op.literal),
        }
    }
}

impl std::fmt::Display for StmtKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use StmtKind::*;
        match self {
            Assign(lhs, rhs, g) => {
                if *g {
                    write!(f, "global {lhs} = {rhs}")
                } else {
                    write!(f, "{lhs} = {rhs}")
                }
            }
            Crawl(lit) => {
                write!(f, "crawl ")?;

                if let Some(bindings) = &lit.bindings {
                    write!(f, "{}, {} ", bindings.link, bindings.depth)?;
                }

                if let Some(filter) = &lit.filter {
                    write!(f, "where {filter}")?;
                }

                write!(f, "do\n{}end\n", lit.body)
            }
            Expr(expr) => write!(f, "{expr}"),
            ForLoop(floop) => {
                write!(
                    f,
                    "for {} in {} do\n{}end\n",
                    floop.ident, floop.iterable, floop.block
                )
            }
            WhileLoop(cond, block) => write!(f, "while {cond} do\n{block}end\n"),
            Func(def, g) => {
                if *g {
                    write!(f, "global {def}")
                } else {
                    write!(f, "{def}")
                }
            }
            Goto(expr) => write!(f, "goto {expr}"),
            IfElse(lit) => {
                writeln!(f, "if {} do\n{}", lit.if_lit.cond, lit.if_lit.block)?;
                for elif in &lit.elifs {
                    writeln!(f, "elif {} do\n{}", elif.cond, elif.block)?;
                }
                if let Some(el) = &lit.else_lit {
                    writeln!(f, "else\n{}", el.block)?;
                }
                writeln!(f, "end")
            }
            Return(mb_expr) => {
                write!(f, "return")?;
                if let Some(expr) = mb_expr {
                    write!(f, "{expr}")?;
                }
                Ok(())
            }
            Scrape(hash) => write!(f, "scrape {hash}"),
            Screenshot(s) => write!(f, "screenshot {s}"),
            TryCatch(t, c) => {
                write!(f, "try\n{t}\n")?;
                if let Some(catch) = c {
                    write!(f, "catch\n{catch}\n")?;
                }
                writeln!(f, "end")
            }
            Use(expr) => write!(f, "use {expr}"),
        }
    }
}

impl std::fmt::Display for Program {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for stmt in &self.stmts {
            writeln!(f, "{stmt}")?;
        }
        Ok(())
    }
}
