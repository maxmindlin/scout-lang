use std::collections::HashMap;

use ast::{
    CrawlBindings, CrawlLiteral, ExprKind, FnParam, ForLoop, FuncDef, HashLiteral, Identifier,
    IfElseLiteral, IfLiteral, Program, StmtKind,
};
use scout_lexer::{Lexer, Token, TokenKind};

use crate::ast::{Block, ElseLiteral};

pub mod ast;

type ParseResult<T> = Result<T, ParseError>;
type PrefixParseFn = fn(parser: &mut Parser) -> ParseResult<ExprKind>;
type InfixParseFn = fn(parser: &mut Parser, ExprKind) -> ParseResult<ExprKind>;

#[derive(Debug, PartialOrd, Ord, PartialEq, Eq)]
enum Precedence {
    Lowest,
    Equals,
    LessGreater,
    Sum,
    Product,
    Call,
    Index,
}

impl From<TokenKind> for Precedence {
    fn from(value: TokenKind) -> Self {
        use TokenKind::*;
        match value {
            And => Self::Equals,
            Or => Self::Equals,
            EQ => Self::Equals,
            NEQ => Self::Equals,
            LT => Self::LessGreater,
            GT => Self::LessGreater,
            LTE => Self::LessGreater,
            GTE => Self::LessGreater,
            Plus => Self::Sum,
            Minus => Self::Sum,
            Slash => Self::Product,
            Asterisk => Self::Product,
            DbColon => Self::Product,
            LParen => Self::Call,
            LBracket => Self::Index,
            Pipe => Self::Index,
            _ => Self::Lowest,
        }
    }
}

fn map_prefix_fn(kind: &TokenKind) -> Option<PrefixParseFn> {
    use TokenKind::*;
    match kind {
        Ident => Some(Parser::parse_ident),
        Int => Some(Parser::parse_number_literal),
        Float => Some(Parser::parse_number_literal),
        True => Some(Parser::parse_boolean),
        False => Some(Parser::parse_boolean),
        Str => Some(Parser::parse_str_literal),
        Null => Some(Parser::parse_null),
        LBracket => Some(Parser::parse_list_literal),
        SelectAll => Some(Parser::parse_select_all),
        Select => Some(Parser::parse_select),
        Bang => Some(Parser::parse_prefix),
        _ => None,
    }
}

fn map_infix_fn(kind: &TokenKind) -> Option<InfixParseFn> {
    use TokenKind::*;
    match kind {
        Plus => Some(Parser::parse_infix),
        Minus => Some(Parser::parse_infix),
        Slash => Some(Parser::parse_infix),
        Asterisk => Some(Parser::parse_infix),
        EQ => Some(Parser::parse_infix),
        NEQ => Some(Parser::parse_infix),
        LT => Some(Parser::parse_infix),
        GT => Some(Parser::parse_infix),
        LTE => Some(Parser::parse_infix),
        GTE => Some(Parser::parse_infix),
        And => Some(Parser::parse_infix),
        Or => Some(Parser::parse_infix),
        DbColon => Some(Parser::parse_infix),
        LBracket => Some(Parser::parse_index),
        LParen => Some(Parser::parse_call_expr),
        Pipe => Some(Parser::parse_chain_expr),
        _ => None,
    }
}

#[derive(Debug)]
pub enum ParseError {
    UnexpectedToken(TokenKind, TokenKind),
    InvalidToken(TokenKind),
    InvalidNumber,
    InvalidFnCall,
    DefaultFnParamBefore,
    UnknownPrefix(TokenKind),
}

pub struct Parser {
    lex: Lexer,
    curr: Token,
    peek: Token,
}

impl Parser {
    pub fn new(mut lex: Lexer) -> Self {
        let curr = lex.next_token();
        let peek = lex.next_token();
        Self { lex, curr, peek }
    }

    pub fn parse_program(&mut self) -> ParseResult<Program> {
        let mut prgm = Program::default();
        while self.curr.kind != TokenKind::EOF {
            let stmt = self.parse_stmt()?;
            prgm.stmts.push(stmt);
            self.next_token();
        }
        Ok(prgm)
    }

    fn next_token(&mut self) {
        let prev = std::mem::replace(&mut self.peek, self.lex.next_token());
        self.curr = prev;
    }

    fn expect_peek(&mut self, expected: TokenKind) -> ParseResult<()> {
        if self.peek.kind == expected {
            self.next_token();
            Ok(())
        } else {
            Err(ParseError::UnexpectedToken(expected, self.peek.kind))
        }
    }

    fn peek_precedence(&self) -> Precedence {
        Precedence::from(self.peek.kind)
    }

    fn curr_precedence(&self) -> Precedence {
        Precedence::from(self.curr.kind)
    }

    fn parse_stmt(&mut self) -> ParseResult<StmtKind> {
        match self.curr.kind {
            TokenKind::Def => {
                self.expect_peek(TokenKind::Ident)?;
                let ident = Identifier::new(self.curr.literal.clone());
                self.expect_peek(TokenKind::LParen)?;

                let mut args = Vec::new();
                let mut has_defaults = false;
                while self.peek.kind == TokenKind::Comma || self.peek.kind != TokenKind::RParen {
                    self.next_token();
                    match self.curr.kind {
                        TokenKind::Comma => {}
                        TokenKind::Ident => {
                            let ident = Identifier::new(self.curr.literal.clone());
                            let mut default = None;
                            if self.peek.kind == TokenKind::Assign {
                                self.next_token();
                                self.next_token();
                                default = Some(self.parse_expr(Precedence::Lowest)?);
                                has_defaults = true;
                            } else if has_defaults {
                                // Dont allow non-default params after default params.
                                // If we dont disallow this then the interpreter will have a
                                // hard time
                                return Err(ParseError::DefaultFnParamBefore);
                            }
                            args.push(FnParam::new(ident, default));
                        }
                        _ => {
                            return Err(ParseError::InvalidToken(self.curr.kind));
                        }
                    }
                }

                self.expect_peek(TokenKind::RParen)?;
                self.expect_peek(TokenKind::Do)?;
                self.next_token();

                let block = self.parse_block(vec![TokenKind::End])?;

                Ok(StmtKind::Func(FuncDef::new(ident, args, block)))
            }
            TokenKind::Goto => self.parse_goto_stmt(),
            TokenKind::Scrape => self.parse_scrape_stmt(),
            TokenKind::For => self.parse_for_loop(),
            TokenKind::While => self.parse_while_loop(),
            TokenKind::Screenshot => self.parse_screenshot_stmt(),
            TokenKind::If => self.parse_if_else(),
            TokenKind::Ident => match self.peek.kind {
                TokenKind::Assign => {
                    let ident = Identifier::new(self.curr.literal.clone());
                    self.next_token();
                    self.next_token();
                    let val = self.parse_expr(Precedence::Lowest)?;
                    Ok(StmtKind::Assign(ident, val))
                }
                _ => self.parse_expr_stmt(),
            },
            TokenKind::Return => {
                self.next_token();
                match self.parse_expr(Precedence::Lowest) {
                    Ok(expr) => Ok(StmtKind::Return(Some(expr))),
                    Err(ParseError::InvalidToken(_)) => Ok(StmtKind::Return(None)),
                    Err(e) => Err(e),
                }
            }
            TokenKind::Use => self.parse_use_stmt(),
            TokenKind::Try => self.parse_try_catch(),
            TokenKind::Crawl => self.parse_crawl(),
            _ => self.parse_expr_stmt(),
        }
    }

    fn parse_crawl(&mut self) -> ParseResult<StmtKind> {
        let mut binding = None;
        if self.peek.kind == TokenKind::Ident {
            self.next_token();
            let link_binding = Identifier::new(self.curr.literal.clone());
            self.expect_peek(TokenKind::Comma)?;
            self.expect_peek(TokenKind::Ident)?;
            let depth_binding = Identifier::new(self.curr.literal.clone());
            binding = Some(CrawlBindings {
                link: link_binding,
                depth: depth_binding,
            });
        }

        let mut filter = None;
        if self.peek.kind == TokenKind::Where {
            self.next_token();
            self.next_token();
            let expr = self.parse_expr(Precedence::Lowest)?;
            filter = Some(expr);
        }
        self.expect_peek(TokenKind::Do)?;
        self.next_token();
        let block = self.parse_block(vec![TokenKind::End])?;
        Ok(StmtKind::Crawl(CrawlLiteral::new(binding, filter, block)))
    }

    fn parse_if_else(&mut self) -> ParseResult<StmtKind> {
        let if_lit = self.parse_if()?;
        let mut elifs = Vec::new();
        while self.curr.kind == TokenKind::Elif {
            elifs.push(self.parse_if()?)
        }
        let mut else_lit = None;
        if self.curr.kind == TokenKind::Else {
            self.next_token();
            let block = self.parse_block(vec![TokenKind::End])?;
            else_lit = Some(ElseLiteral { block })
        }

        Ok(StmtKind::IfElse(IfElseLiteral {
            if_lit,
            elifs,
            else_lit,
        }))
    }

    fn parse_if(&mut self) -> ParseResult<IfLiteral> {
        self.next_token();
        let cond = self.parse_expr(Precedence::Lowest)?;
        self.expect_peek(TokenKind::Do)?;
        self.next_token();
        let block = self.parse_block(vec![TokenKind::End, TokenKind::Elif, TokenKind::Else])?;
        Ok(IfLiteral { cond, block })
    }

    fn parse_prefix(&mut self) -> ParseResult<ExprKind> {
        let op = self.curr.kind;
        self.next_token();
        let expr = self.parse_expr(Precedence::Lowest)?;
        Ok(ExprKind::Prefix(Box::new(expr), op))
    }

    fn parse_block(&mut self, finalizers: Vec<TokenKind>) -> ParseResult<Block> {
        let mut stmts = Vec::new();
        while !finalizers.contains(&self.curr.kind) {
            let stmt = self.parse_stmt()?;
            stmts.push(stmt);
            self.next_token();
        }
        Ok(Block::new(stmts))
    }

    fn parse_try_catch(&mut self) -> ParseResult<StmtKind> {
        self.next_token();
        let try_b = self.parse_block(vec![TokenKind::Catch, TokenKind::End])?;
        let catch_b = if self.curr.kind == TokenKind::Catch {
            self.next_token();
            let block = self.parse_block(vec![TokenKind::End])?;
            Some(block)
        } else {
            None
        };
        // self.next_token();
        // let catch_b = self.parse_block(vec![TokenKind::End])?;

        Ok(StmtKind::TryCatch(try_b, catch_b))
    }

    fn parse_while_loop(&mut self) -> ParseResult<StmtKind> {
        self.next_token();
        let condition = self.parse_expr(Precedence::Lowest)?;
        self.expect_peek(TokenKind::Do)?;
        self.next_token();
        let block = self.parse_block(vec![TokenKind::End])?;
        Ok(StmtKind::WhileLoop(condition, block))
    }

    /// `for <ident> in <expr> do <block> end`
    fn parse_for_loop(&mut self) -> ParseResult<StmtKind> {
        self.expect_peek(TokenKind::Ident)?;
        let ident = Identifier::new(self.curr.literal.clone());
        self.expect_peek(TokenKind::In)?;
        self.next_token();
        let iterable = self.parse_expr(Precedence::Lowest)?;
        self.expect_peek(TokenKind::Do)?;
        self.next_token();
        let block = self.parse_block(vec![TokenKind::End])?;

        self.next_token();
        let floop = ForLoop::new(ident, iterable, block);
        Ok(StmtKind::ForLoop(floop))
    }

    fn parse_use_stmt(&mut self) -> ParseResult<StmtKind> {
        self.next_token();
        let import = self.parse_expr(Precedence::Lowest)?;
        Ok(StmtKind::Use(import))
    }

    /// `goto "https://stackoverflow.com"`
    fn parse_goto_stmt(&mut self) -> ParseResult<StmtKind> {
        self.next_token();
        let url = self.parse_expr(Precedence::Lowest)?;
        let stmt = StmtKind::Goto(url);
        Ok(stmt)
    }

    /// `screenshot "screenshot.png"`
    fn parse_screenshot_stmt(&mut self) -> ParseResult<StmtKind> {
        self.expect_peek(TokenKind::Str)?;
        let stmt = StmtKind::Screenshot(self.curr.literal.clone());
        Ok(stmt)
    }

    fn parse_expr_stmt(&mut self) -> ParseResult<StmtKind> {
        let expr = self.parse_expr(Precedence::Lowest)?;
        Ok(StmtKind::Expr(expr))
    }

    /// `scrape { body: ".body" }`
    fn parse_scrape_stmt(&mut self) -> ParseResult<StmtKind> {
        self.expect_peek(TokenKind::LBrace)?;
        let body = self.parse_hash_literal()?;
        Ok(StmtKind::Scrape(body))
    }

    /// `{ a: "b", c: "d" }`
    /// @TODO: allow expressions as values instead of strings
    ///
    /// Current token entering should be a LBrace
    fn parse_hash_literal(&mut self) -> ParseResult<HashLiteral> {
        let mut pairs = HashMap::new();
        while self.peek.kind != TokenKind::RBrace {
            self.expect_peek(TokenKind::Ident)?;
            let ident = Identifier::new(self.curr.literal.clone());
            self.expect_peek(TokenKind::Colon)?;
            self.next_token();
            let val = self.parse_expr(Precedence::Lowest)?;
            pairs.insert(ident, val);
            if self.peek.kind == TokenKind::Comma {
                self.next_token();
            }
        }
        self.next_token();
        Ok(HashLiteral { pairs })
    }

    fn parse_number_literal(&mut self) -> ParseResult<ExprKind> {
        Ok(ExprKind::Number(
            self.curr
                .literal
                .parse::<f64>()
                .map_err(|_| ParseError::InvalidNumber)?,
        ))
    }

    fn parse_boolean(&mut self) -> ParseResult<ExprKind> {
        let val = self.curr.kind == TokenKind::True;
        Ok(ExprKind::Boolean(val))
    }

    fn parse_str_literal(&mut self) -> ParseResult<ExprKind> {
        Ok(ExprKind::Str(self.curr.literal.clone()))
    }

    fn parse_null(&mut self) -> ParseResult<ExprKind> {
        Ok(ExprKind::Null)
    }

    fn parse_list_literal(&mut self) -> ParseResult<ExprKind> {
        self.next_token();
        let mut content = vec![];
        while let Ok(expr) = self.parse_expr(Precedence::Lowest) {
            content.push(expr);
            self.next_token();
            if self.curr.kind == TokenKind::Comma {
                self.next_token();
            }
        }

        Ok(ExprKind::List(content))
    }

    fn parse_select(&mut self) -> ParseResult<ExprKind> {
        match self.peek.kind {
            TokenKind::Str => {
                self.next_token();
                Ok(ExprKind::Select(self.curr.literal.clone(), None))
            }
            TokenKind::LParen => {
                // @TODO refactor
                self.next_token();
                self.expect_peek(TokenKind::Ident)?;
                let ident = Identifier::new(self.curr.literal.clone());
                self.expect_peek(TokenKind::RParen)?;
                self.expect_peek(TokenKind::Str)?;
                let expr = ExprKind::Select(self.curr.literal.clone(), Some(ident));
                Ok(expr)
            }
            _ => Err(ParseError::InvalidToken(self.peek.kind)),
        }
    }

    fn parse_select_all(&mut self) -> ParseResult<ExprKind> {
        match self.peek.kind {
            TokenKind::Str => {
                self.next_token();
                Ok(ExprKind::SelectAll(self.curr.literal.clone(), None))
            }
            TokenKind::LParen => {
                // @TODO refactor
                self.next_token();
                self.expect_peek(TokenKind::Ident)?;
                let ident = Identifier::new(self.curr.literal.clone());
                self.expect_peek(TokenKind::RParen)?;
                self.expect_peek(TokenKind::Str)?;
                let expr = ExprKind::SelectAll(self.curr.literal.clone(), Some(ident));
                Ok(expr)
            }
            _ => Err(ParseError::InvalidToken(self.peek.kind)),
        }
    }

    fn parse_infix(&mut self, lhs: ExprKind) -> ParseResult<ExprKind> {
        // self.next_token();
        let op = self.curr.kind;
        let prec = self.curr_precedence();
        self.next_token();
        let rhs = self.parse_expr(prec)?;
        Ok(ExprKind::Infix(Box::new(lhs), op, Box::new(rhs)))
    }

    fn parse_index(&mut self, ident: ExprKind) -> ParseResult<ExprKind> {
        let infix = self.parse_infix(ident)?;
        self.expect_peek(TokenKind::RBracket)?;
        Ok(infix)
    }

    fn parse_expr(&mut self, precedence: Precedence) -> ParseResult<ExprKind> {
        match map_prefix_fn(&self.curr.kind) {
            None => Err(ParseError::UnknownPrefix(self.curr.kind)),
            Some(f) => {
                let mut lhs = f(self)?;
                while precedence < self.peek_precedence() {
                    match map_infix_fn(&self.peek.kind) {
                        None => return Ok(lhs),
                        Some(in_fn) => {
                            self.next_token();
                            lhs = in_fn(self, lhs)?;
                        }
                    }
                }

                Ok(lhs)
            }
        }
    }

    fn parse_ident(&mut self) -> ParseResult<ExprKind> {
        Ok(ExprKind::Ident(Identifier::new(self.curr.literal.clone())))
    }

    fn parse_chain_expr(&mut self, first: ExprKind) -> ParseResult<ExprKind> {
        let mut exprs = vec![first];

        while self.curr.kind == TokenKind::Pipe {
            self.next_token();
            let id = self.parse_ident()?;
            self.expect_peek(TokenKind::LParen)?;
            let call = self.parse_call_expr(id)?;
            exprs.push(call);
        }

        Ok(ExprKind::Chain(exprs))
    }

    fn parse_expr_list(&mut self, end: TokenKind) -> ParseResult<Vec<ExprKind>> {
        let mut args: Vec<ExprKind> = Vec::new();
        if self.peek.kind == end {
            self.next_token();
            return Ok(args);
        }

        self.next_token();
        let expr = self.parse_expr(Precedence::Lowest)?;
        args.push(expr);

        while self.peek.kind == TokenKind::Comma {
            self.next_token();
            self.next_token();
            let e = self.parse_expr(Precedence::Lowest)?;
            args.push(e);
        }

        self.expect_peek(end)?;
        Ok(args)
    }

    fn parse_call_expr(&mut self, func: ExprKind) -> ParseResult<ExprKind> {
        match func {
            ExprKind::Ident(ident) => {
                let args = self.parse_expr_list(TokenKind::RParen)?;
                Ok(ExprKind::Call(ident, args))
            }
            _ => Err(ParseError::InvalidFnCall),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_case::test_case;

    fn setup_parser(i: &str) -> Parser {
        Parser::new(Lexer::new(i))
    }

    fn parse_stmts(i: &str) -> Vec<StmtKind> {
        let mut p = setup_parser(i);
        let prg = p.parse_program().unwrap();
        prg.stmts
    }

    fn extract_first_stmt(i: &str) -> StmtKind {
        let stmts = parse_stmts(i);
        stmts[0].clone()
    }

    #[test_case(r#"goto "foo""#, StmtKind::Goto(ExprKind::Str("foo".into())); "simple goto")]
    #[test_case("scrape {}", StmtKind::Scrape(HashLiteral::default()); "empty scrape")]
    #[test_case(
        r#"scrape { a: $"b" }"#,
        StmtKind::Scrape(
            HashLiteral::from(
                vec![
                    (Identifier::new("a".into()), ExprKind::Select("b".into(), None))
                ]
            )
        ); "scrape with single key"
    )]
    #[test_case(
        r#"scrape { a: $"b", c: $"d" }"#,
        StmtKind::Scrape(
            HashLiteral::from(
                vec![
                    (Identifier::new("a".into()), ExprKind::Select("b".into(), None)),
                    (Identifier::new("c".into()), ExprKind::Select("d".into(), None))
                ]
            )
        ); "scrape with multi keys"
    )]
    #[test_case(
        r#"scrape { a: $$"b" }"#,
        StmtKind::Scrape(
            HashLiteral::from(
                vec![
                    (Identifier::new("a".into()), ExprKind::SelectAll("b".into(), None)),
                ]
            )
        ); "scrape with select all key"
    )]
    #[test_case(
        r#"scrape { a: fn("a") }"#,
        StmtKind::Scrape(
            HashLiteral::from(
                vec![
                    (
                        Identifier::new("a".into()),
                        ExprKind::Call(
                            Identifier::new("fn".into()),
                            vec![ExprKind::Str("a".into())]
                        )
                    )
                ]
            )
        ); "scrape with fn key"
    )]
    #[test_case(
        r#"scrape { a: $"b" |> fn("a") }"#,
        StmtKind::Scrape(
            HashLiteral::from(
                vec![
                    (
                        Identifier::new("a".into()),
                        ExprKind::Chain(vec![
                            ExprKind::Select("b".into(), None),
                            ExprKind::Call(
                                Identifier::new("fn".into()),
                                vec![ExprKind::Str("a".into())]
                            )
                        ])
                    )
                ]
            )
        ); "scrape with pipe key"
    )]
    #[test_case(
        r#"for node in $$"a" do end"#,
        StmtKind::ForLoop(
            ForLoop::new(Identifier::new("node".into()), ExprKind::SelectAll("a".into(), None), Block::new(vec![]))
        ); "for loop empty body"
    )]
    #[test_case(
        r#"for node in $$"a" do $"a" end"#,
        StmtKind::ForLoop(
            ForLoop::new(Identifier::new("node".into()), ExprKind::SelectAll("a".into(), None), Block::new(vec![
                StmtKind::Expr(ExprKind::Select("a".into(), None))
            ]))
        ); "for loop single select bodyd"
    )]
    #[test_case(
        r#"x = "a""#,
        StmtKind::Assign(
            Identifier::new("x".into()),
            ExprKind::Str("a".into())
        ); "single assign"
    )]
    #[test_case(r#"null"#, StmtKind::Expr(ExprKind::Null); "null expr stmt")]
    #[test_case(
        r#"for node in $$"a" do scrape {} end"#,
        StmtKind::ForLoop(
            ForLoop::new(Identifier::new("node".into()), ExprKind::SelectAll("a".into(), None), Block::new(vec![
                StmtKind::Scrape(HashLiteral::default())
            ]))
        ); "for loop with scrape body"
    )]
    #[test_case(
        r#"x = 1 == 2"#,
        StmtKind::Assign(
            Identifier::new("x".to_string()),
            ExprKind::Infix(Box::new(ExprKind::Number(1.)), TokenKind::EQ, Box::new(ExprKind::Number(2.)))
        ); "assign eq infix"
    )]
    #[test_case(
        r#"f(a, b)"#,
        StmtKind::Expr(
            ExprKind::Call(
                Identifier::new("f".into()),
                vec![
                    ExprKind::Ident(Identifier::new("a".into())),
                    ExprKind::Ident(Identifier::new("b".into()))
                ]
            )
        ); "fn call with multi params"
    )]
    #[test_case(
        r#"def f() do end"#,
        StmtKind::Func(
            FuncDef::new(
                Identifier::new("f".into()),
                vec![
                ],
                Block::default()
            )
        ); "fn definition"
    )]
    #[test_case(
        r#"def f(a, b) do end"#,
        StmtKind::Func(
            FuncDef::new(
                Identifier::new("f".into()),
                vec![
                    FnParam::new(Identifier::new("a".into()), None),
                    FnParam::new(Identifier::new("b".into()), None)
                ],
                Block::default()
            )
        ); "fn def multi params"
    )]
    #[test_case(
        r#"def f(a = null) do end"#,
        StmtKind::Func(
            FuncDef::new(
                Identifier::new("f".into()),
                vec![
                    FnParam::new(Identifier::new("a".into()), Some(ExprKind::Null))
                ],
                Block::default()
            )
        ); "fn def default param"
    )]
    #[test_case(
        r#"[1, "a"]"#,
        StmtKind::Expr(
            ExprKind::List(
                vec![
                    ExprKind::Number(1.0),
                    ExprKind::Str("a".into()),
                ]
            )
        ); "list literal"
    )]
    #[test_case(
        r#"for a in [1, 2] do end"#,
        StmtKind::ForLoop(
            ForLoop::new(Identifier::new("a".into()), ExprKind::List(vec![
                ExprKind::Number(1.0),
                ExprKind::Number(2.0),
            ]), Block::new(vec![]))
        ); "loop over list literal"
    )]
    #[test_case(
        "try catch end",
        StmtKind::TryCatch(Block::default(), Some(Block::default())); "empty try catch"
    )]
    #[test_case("try end", StmtKind::TryCatch(Block::default(), None); "try catch with no catch")]
    #[test_case(
        "a[0]",
        StmtKind::Expr(
            ExprKind::Infix(
                Box::new(ExprKind::Ident(Identifier::new("a".into()))),
                TokenKind::LBracket,
                Box::new(ExprKind::Number(0.))
            )
        ); "index"
    )]
    #[test_case(
        "crawl do end",
        StmtKind::Crawl(CrawlLiteral::new(None, None, Block::default())); "empty crawl stmtddddd"
    )]
    #[test_case(
        "crawl link, depth where depth < 1 do end",
        StmtKind::Crawl(
            CrawlLiteral::new(
                Some(CrawlBindings {
                    link: Identifier::new("link".into()),
                    depth: Identifier::new("depth".into())
                }),
                Some(ExprKind::Infix(Box::new(ExprKind::Ident(Identifier::new("depth".into()))), TokenKind::LT, Box::new(ExprKind::Number(1.)))),
                Block::default()
            )
        ); "crawl stmt with bindings"
    )]
    #[test_case(
        "!true",
        StmtKind::Expr(ExprKind::Prefix(Box::new(ExprKind::Boolean(true)), TokenKind::Bang,)); "bang prefix"
    )]
    #[test_case(
        "a::b",
        StmtKind::Expr(
            ExprKind::Infix(
                Box::new(
                    ExprKind::Ident(Identifier::new("a".into()))
                ),
                TokenKind::DbColon,
                Box::new(
                    ExprKind::Ident(Identifier::new("b".into()))
                )
            )
        ); "db colon"
    )]
    #[test_case(
        "while a < 1 do end",
        StmtKind::WhileLoop(
            ExprKind::Infix(
                Box::new(
                    ExprKind::Ident(Identifier::new("a".into()))
                ),
                TokenKind::LT,
                Box::new(
                    ExprKind::Number(1.)
                )
            ),
            Block::default(),
        ); "while loop"
    )]
    fn test_single_stmt(input: &str, exp: StmtKind) {
        let stmt = extract_first_stmt(input);
        assert_eq!(stmt, exp);
    }

    #[test]
    fn test_if_else() {
        let input = r#"if 1 do elif 2 do else end"#;
        let stmt = extract_first_stmt(input);
        let exp = StmtKind::IfElse(IfElseLiteral {
            if_lit: IfLiteral {
                cond: ExprKind::Number(1.),
                block: Block::default(),
            },
            elifs: vec![IfLiteral {
                cond: ExprKind::Number(2.),
                block: Block::default(),
            }],
            else_lit: Some(ElseLiteral {
                block: Block::default(),
            }),
        });
        assert_eq!(stmt, exp);
    }
}
