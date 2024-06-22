use std::collections::HashMap;

use ast::{
    ExprKind, ForLoop, FuncDef, HashLiteral, Identifier, IfElseLiteral, IfLiteral, Program,
    StmtKind,
};
use scout_lexer::{Lexer, Token, TokenKind};

use crate::ast::{Block, ElseLiteral};

pub mod ast;

type ParseResult<T> = Result<T, ParseError>;

#[derive(Debug)]
pub enum ParseError {
    UnexpectedToken(TokenKind, TokenKind),
    InvalidToken(TokenKind),
    InvalidNumber,
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

    fn parse_stmt(&mut self) -> ParseResult<StmtKind> {
        match self.curr.kind {
            TokenKind::Def => {
                self.expect_peek(TokenKind::Ident)?;
                let ident = Identifier::new(self.curr.literal.clone());
                self.expect_peek(TokenKind::LParen)?;

                let mut args = Vec::new();
                while self.peek.kind == TokenKind::Comma || self.peek.kind != TokenKind::RParen {
                    self.next_token();
                    match self.curr.kind {
                        TokenKind::Comma => {}
                        TokenKind::Ident => {
                            args.push(Identifier::new(self.curr.literal.clone()));
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
            TokenKind::Screenshot => self.parse_screenshot_stmt(),
            TokenKind::If => self.parse_if_else(),
            TokenKind::Ident => match self.peek.kind {
                TokenKind::Assign => {
                    let ident = Identifier::new(self.curr.literal.clone());
                    self.next_token();
                    self.next_token();
                    let val = self.parse_expr()?;
                    Ok(StmtKind::Assign(ident, val))
                }
                _ => self.parse_expr_stmt(),
            },
            TokenKind::Return => {
                self.next_token();
                match self.parse_expr() {
                    Ok(expr) => Ok(StmtKind::Return(Some(expr))),
                    Err(ParseError::InvalidToken(_)) => Ok(StmtKind::Return(None)),
                    Err(e) => Err(e),
                }
            }
            TokenKind::Use => self.parse_use_stmt(),
            TokenKind::Try => self.parse_try_catch(),
            _ => self.parse_expr_stmt(),
        }
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
        let cond = self.parse_expr()?;
        self.expect_peek(TokenKind::Do)?;
        self.next_token();
        let block = self.parse_block(vec![TokenKind::End, TokenKind::Elif, TokenKind::Else])?;
        Ok(IfLiteral { cond, block })
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

    /// `for <ident> in <expr> do <block> end`
    fn parse_for_loop(&mut self) -> ParseResult<StmtKind> {
        self.expect_peek(TokenKind::Ident)?;
        let ident = Identifier::new(self.curr.literal.clone());
        self.expect_peek(TokenKind::In)?;
        self.next_token();
        let iterable = self.parse_expr()?;
        self.expect_peek(TokenKind::Do)?;
        self.next_token();
        let block = self.parse_block(vec![TokenKind::End])?;

        self.next_token();
        let floop = ForLoop::new(ident, iterable, block);
        Ok(StmtKind::ForLoop(floop))
    }

    fn parse_use_stmt(&mut self) -> ParseResult<StmtKind> {
        self.next_token();
        let import = self.parse_expr()?;
        Ok(StmtKind::Use(import))
    }

    /// `goto "https://stackoverflow.com"`
    fn parse_goto_stmt(&mut self) -> ParseResult<StmtKind> {
        self.next_token();
        let url = self.parse_expr()?;
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
        let expr = self.parse_expr()?;
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
            let val = self.parse_expr()?;
            pairs.insert(ident, val);
            if self.peek.kind == TokenKind::Comma {
                self.next_token();
            }
        }
        self.next_token();
        Ok(HashLiteral { pairs })
    }

    fn parse_single_expr(&mut self) -> ParseResult<ExprKind> {
        let lhs = match self.curr.kind {
            TokenKind::False => Ok(ExprKind::Boolean(false)),
            TokenKind::True => Ok(ExprKind::Boolean(true)),
            TokenKind::Ident => {
                // Parse multiple types of ident expressions
                match self.peek.kind {
                    TokenKind::LParen => self.parse_call_expr(),
                    _ => Ok(ExprKind::Ident(Identifier::new(self.curr.literal.clone()))),
                }
            }
            TokenKind::LBracket => {
                self.next_token();
                let mut content = vec![];
                while let Ok(expr) = self.parse_expr() {
                    content.push(expr);
                    self.next_token();
                    if self.curr.kind == TokenKind::Comma {
                        self.next_token();
                    }
                }

                Ok(ExprKind::List(content))
            }
            TokenKind::Select => match self.peek.kind {
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
            },
            TokenKind::SelectAll => match self.peek.kind {
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
            },
            TokenKind::Str => Ok(ExprKind::Str(self.curr.literal.clone())),
            TokenKind::Int => Ok(ExprKind::Number(
                self.curr
                    .literal
                    .parse::<u64>()
                    .map_err(|_| ParseError::InvalidNumber)? as f64,
            )),
            TokenKind::Float => Ok(ExprKind::Number(
                self.curr
                    .literal
                    .parse()
                    .map_err(|_| ParseError::InvalidNumber)?,
            )),
            TokenKind::Null => Ok(ExprKind::Null),
            _ => Err(ParseError::InvalidToken(self.curr.kind)),
        }?;

        if self.peek.kind.is_infix() {
            self.next_token();
            let op = self.curr.kind;
            self.next_token();
            let rhs = self.parse_expr()?;
            Ok(ExprKind::Infix(Box::new(lhs), op, Box::new(rhs)))
        } else {
            Ok(lhs)
        }
    }

    fn parse_expr(&mut self) -> ParseResult<ExprKind> {
        let expr = self.parse_single_expr()?;

        if self.peek.kind == TokenKind::Pipe {
            let mut exprs = vec![expr];

            while self.peek.kind == TokenKind::Pipe {
                self.next_token();
                self.next_token();
                let next_expr = self.parse_single_expr()?;
                exprs.push(next_expr);
            }

            Ok(ExprKind::Chain(exprs))
        } else {
            Ok(expr)
        }
    }

    fn parse_call_expr(&mut self) -> ParseResult<ExprKind> {
        let ident = Identifier::new(self.curr.literal.clone());
        self.next_token();
        let mut params = Vec::new();
        while self.peek.kind == TokenKind::Comma || self.peek.kind != TokenKind::RParen {
            self.next_token();
            if self.curr.kind == TokenKind::Comma {
                self.next_token();
            }
            let param = self.parse_expr()?;
            params.push(param);
        }

        self.expect_peek(TokenKind::RParen)?;
        Ok(ExprKind::Call(ident, params))
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

    #[test_case(r#"goto "foo""#, StmtKind::Goto(ExprKind::Str("foo".into())))]
    #[test_case("scrape {}", StmtKind::Scrape(HashLiteral::default()))]
    #[test_case(
        r#"scrape { a: $"b" }"#,
        StmtKind::Scrape(
            HashLiteral::from(
                vec![
                    (Identifier::new("a".into()), ExprKind::Select("b".into(), None))
                ]
            )
        )
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
        )
    )]
    #[test_case(
        r#"scrape { a: $$"b" }"#,
        StmtKind::Scrape(
            HashLiteral::from(
                vec![
                    (Identifier::new("a".into()), ExprKind::SelectAll("b".into(), None)),
                ]
            )
        )
    )]
    #[test_case(
        r#"scrape { a: fn("a") }"#,
        StmtKind::Scrape(
            HashLiteral::from(
                vec![
                    (
                        Identifier::new("a".into()),
                        ExprKind::Call(
                            Identifier::new("fn".into()), vec![ExprKind::Str("a".into())]
                        )
                    )
                ]
            )
        )
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
        )
    )]
    #[test_case(
        r#"for node in $$"a" do end"#,
        StmtKind::ForLoop(
            ForLoop::new(Identifier::new("node".into()), ExprKind::SelectAll("a".into(), None), Block::new(vec![]))
        )
    )]
    #[test_case(
        r#"for node in $$"a" do $"a" end"#,
        StmtKind::ForLoop(
            ForLoop::new(Identifier::new("node".into()), ExprKind::SelectAll("a".into(), None), Block::new(vec![
                StmtKind::Expr(ExprKind::Select("a".into(), None))
            ]))
        )
    )]
    #[test_case(
        r#"x = "a""#,
        StmtKind::Assign(
            Identifier::new("x".into()),
            ExprKind::Str("a".into())
        )
    )]
    #[test_case(
        r#"for node in $$"a" do scrape {} end"#,
        StmtKind::ForLoop(
            ForLoop::new(Identifier::new("node".into()), ExprKind::SelectAll("a".into(), None), Block::new(vec![
                StmtKind::Scrape(HashLiteral::default())
            ]))
        )
    )]
    #[test_case(
        r#"x = 1 == 2"#,
        StmtKind::Assign(
            Identifier::new("x".to_string()),
            ExprKind::Infix(Box::new(ExprKind::Number(1.)), TokenKind::EQ, Box::new(ExprKind::Number(2.)))
        )
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
        )
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
        )
    )]
    #[test_case(
        r#"def f(a, b) do end"#,
        StmtKind::Func(
            FuncDef::new(
                Identifier::new("f".into()),
                vec![
                    Identifier::new("a".into()),
                    Identifier::new("b".into())
                ],
                Block::default()
            )
        )
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
        )
    )]
    #[test_case(
        r#"for a in [1, 2] do end"#,
        StmtKind::ForLoop(
            ForLoop::new(Identifier::new("a".into()), ExprKind::List(vec![
                ExprKind::Number(1.0),
                ExprKind::Number(2.0),
            ]), Block::new(vec![]))
        )
    )]
    #[test_case(
        "try catch end",
        StmtKind::TryCatch(Block::default(), Some(Block::default()))
    )]
    #[test_case("try end", StmtKind::TryCatch(Block::default(), None))]
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
