use std::collections::HashMap;

use ast::{ExprKind, HashLiteral, Identifier, Program, StmtKind};
use scout_lexer::{Lexer, Token, TokenKind};

pub mod ast;

type ParseResult<T> = Result<T, ParseError>;

#[derive(Debug)]
pub enum ParseError {
    UnexpectedToken(TokenKind, TokenKind),
    InvalidToken(TokenKind),
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
            TokenKind::Goto => self.parse_goto_stmt(),
            TokenKind::Scrape => self.parse_scrape_stmt(),
            _ => self.parse_expr_stmt(),
        }
    }

    /// `goto "https://stackoverflow.com"`
    fn parse_goto_stmt(&mut self) -> ParseResult<StmtKind> {
        self.expect_peek(TokenKind::Str)?;
        let stmt = StmtKind::Goto(self.curr.literal.clone());
        self.next_token();
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
        self.next_token();
        Ok(HashLiteral { pairs })
    }

    fn parse_single_expr(&mut self) -> ParseResult<ExprKind> {
        match self.curr.kind {
            TokenKind::Ident => {
                // Parse multiple types of ident expressions
                match self.peek.kind {
                    TokenKind::LParen => self.parse_call_expr(),
                    _ => Err(ParseError::InvalidToken(self.peek.kind)),
                }
            }
            TokenKind::Select => Ok(ExprKind::Select(self.curr.literal.clone())),
            TokenKind::Str => Ok(ExprKind::Str(self.curr.literal.clone())),
            _ => Err(ParseError::InvalidToken(self.curr.kind)),
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

    #[test_case(r#"goto "foo""#, StmtKind::Goto("foo".into()))]
    #[test_case("scrape {}", StmtKind::Scrape(HashLiteral::default()))]
    #[test_case(
        r#"scrape { a: $"b" }"#,
        StmtKind::Scrape(
            HashLiteral::from(
                vec![
                    (Identifier::new("a".into()), ExprKind::Select("b".into()))
                ]
            )
        )
    )]
    #[test_case(
        r#"scrape { a: $"b", c: $"d" }"#,
        StmtKind::Scrape(
            HashLiteral::from(
                vec![
                    (Identifier::new("a".into()), ExprKind::Select("b".into())),
                    (Identifier::new("c".into()), ExprKind::Select("d".into()))
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
                            ExprKind::Select("b".into()),
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
    fn test_single_stmt(input: &str, exp: StmtKind) {
        let stmt = extract_first_stmt(input);
        assert_eq!(stmt, exp);
    }
}
