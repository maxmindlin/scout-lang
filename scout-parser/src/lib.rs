use ast::{Program, StmtKind};
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

    fn parse_goto_stmt(&mut self) -> ParseResult<StmtKind> {
        self.expect_peek(TokenKind::Str)?;
        let stmt = StmtKind::Goto(self.curr.literal.clone());
        self.next_token();
        Ok(stmt)
    }

    fn parse_scrape_stmt(&mut self) -> ParseResult<StmtKind> {
        self.expect_peek(TokenKind::LBrace)?;
        while self.peek.kind != TokenKind::RBrace {
            match self.peek.kind {
                TokenKind::Ident => {
                    self.expect_peek(TokenKind::Colon)?;
                    self.expect_peek(TokenKind::Str)?;
                }
                _ => return Err(ParseError::InvalidToken(self.peek.kind)),
            };
        }
        self.next_token();
        self.next_token();
        Ok(StmtKind::Scrape)
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
    #[test_case("scrape {}", StmtKind::Scrape)]
    #[test_case(r#"scrape { a: "b"}"#, StmtKind::Scrape)]
    fn test_single_stmt(input: &str, exp: StmtKind) {
        let stmt = extract_first_stmt(input);
        assert_eq!(stmt, exp);
    }
}

