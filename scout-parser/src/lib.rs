use std::{collections::HashMap, hash::Hash};

use ast::{HashLiteral, Identifier, Program, StmtKind};
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
            _ => Err(ParseError::InvalidToken(self.curr.kind)),
        }
    }

    /// `goto "https://stackoverflow.com"`
    fn parse_goto_stmt(&mut self) -> ParseResult<StmtKind> {
        self.expect_peek(TokenKind::Str)?;
        let stmt = StmtKind::Goto(self.curr.literal.clone());
        self.next_token();
        Ok(stmt)
    }

    /// `scrape { body: ".body" }`
    fn parse_scrape_stmt(&mut self) -> ParseResult<StmtKind> {
        self.expect_peek(TokenKind::LBrace)?;
        let body = self.parse_hash_literal()?;
        self.next_token();
        self.next_token();
        Ok(StmtKind::Scrape(body))
    }

    /// `{ a: "b", c: "d" }`
    fn parse_hash_literal(&mut self) -> ParseResult<HashLiteral> {
        let pairs = HashMap::new();
        while self.peek.kind != TokenKind::RBrace {
            self.expect_peek(TokenKind::Ident)?;
            let ident = Identifier::new(self.curr.literal.clone());
            self.expect_peek(TokenKind::Colon)?;
            self.expect_peek(TokenKind::Str)?;
        }
        Ok(HashLiteral { pairs })
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
    #[test_case(r#"scrape { a: "b" }"#, StmtKind::Scrape(HashLiteral::default()))]
    fn test_single_stmt(input: &str, exp: StmtKind) {
        let stmt = extract_first_stmt(input);
        assert_eq!(stmt, exp);
    }
}

