pub mod token;

pub use token::{Token, TokenKind};

#[derive(Default)]
pub struct Lexer {
    input: Vec<char>,
    pos: usize,
    read_pos: usize,
}

impl Lexer {
    pub fn new(input: &str) -> Self {
        Self {
            input: input.chars().collect(),
            ..Default::default()
        }
    }

    pub fn next_token(&mut self) -> Token {
        use TokenKind::*;
        match self.next() {
            Some(c) => match *c {
                '(' => Token::new(LParen, c.to_string()),
                ')' => Token::new(RParen, c.to_string()),
                '{' => Token::new(LBrace, c.to_string()),
                '}' => Token::new(RBrace, c.to_string()),
                ':' => Token::new(Colon, c.to_string()),
                ',' => Token::new(Comma, c.to_string()),
                '"' => {
                    let literal = self.read_string();
                    Token::new(Str, literal)
                }
                '|' => match self.peek() {
                    Some(pc) => match *pc {
                        '>' => {
                            let mut lit = '|'.to_string();
                            lit.push(*self.next().unwrap());
                            Token::new(Pipe, lit)
                        }
                        _ => Token::new(Illegal, pc.to_string()),
                    },
                    None => Token::new(Illegal, '|'.to_string()),
                },
                '$' => match self.peek() {
                    Some(c) => match *c {
                        '"' => {
                            let _ = self.next();
                            let literal = self.read_string();
                            Token::new(Select, literal)
                        }
                        '$' => {
                            let _ = self.next();
                            match self.peek() {
                                Some(c) => match *c {
                                    '"' => {
                                        let _ = self.next();
                                        let literal = self.read_string();
                                        Token::new(SelectAll, literal)
                                    }
                                    _ => Token::new(Illegal, c.to_string()),
                                },
                                _ => Token::new(Illegal, "$$".to_string()),
                            }
                        }
                        _ => Token::new(Illegal, c.to_string()),
                    },
                    _ => Token::new(Illegal, '$'.to_string()),
                },
                _ if c.is_whitespace() => self.next_token(),
                _ if c.is_numeric() => {
                    let lit = self.read_numeric();
                    if lit.contains('.') {
                        Token::new(Float, lit)
                    } else {
                        Token::new(Int, lit)
                    }
                }
                _ if c.is_alphabetic() => {
                    let lit = self.read_identifier();
                    match TokenKind::is_to_keyword(&lit) {
                        Some(t) => Token::new(t, lit),
                        None => Token::new(Ident, lit),
                    }
                }
                _ => Token::new(Illegal, c.to_string()),
            },
            None => Token::new(EOF, "".into()),
        }
    }

    fn next(&mut self) -> Option<&char> {
        let out = self.input.get(self.read_pos);
        self.pos = self.read_pos;
        self.read_pos += 1;
        out
    }

    fn peek(&mut self) -> Option<&char> {
        self.input.get(self.read_pos)
    }

    fn read_identifier(&mut self) -> String {
        let start = self.pos;
        let mut i = vec![self.input[start]];
        while self.peek().is_some() && is_valid_identifier(self.peek().unwrap()) {
            i.push(*self.next().unwrap());
        }
        i.iter().collect()
    }

    fn read_numeric(&mut self) -> String {
        let start = self.pos;
        let mut i = vec![self.input[start]];
        while self.peek().is_some() && is_valid_numeric(self.peek().unwrap()) {
            i.push(*self.next().unwrap());
        }
        i.iter().collect()
    }

    fn read_string(&mut self) -> String {
        let mut i: Vec<char> = Vec::new();
        loop {
            match self.peek() {
                Some(c) if *c == '"' => {
                    let _ = self.next();
                    break;
                }
                Some(_) => {
                    i.push(*self.next().unwrap());
                }
                None => {
                    break;
                }
            };
        }
        i.iter().collect()
    }
}

fn is_valid_identifier(c: &char) -> bool {
    c.is_alphabetic() || c.is_numeric() || *c == '_'
}

fn is_valid_numeric(c: &char) -> bool {
    c.is_numeric() || *c == '.'
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_case::test_case;
    use TokenKind::*;

    #[test_case("for", vec!(Token::new(For, "for".into())))]
    #[test_case("x", vec!(Token::new(Ident, "x".into())))]
    #[test_case("for x", vec!(Token::new(For, "for".into()), Token::new(Ident, "x".into())))]
    #[test_case("\"x\"", vec!(Token::new(Str, "x".into())))]
    #[test_case("|>", vec!(Token::new(Pipe, "|>".into())))]
    #[test_case(r#"$".div" $"a""#, vec!(Token::new(Select, ".div".into()), Token::new(Select, "a".into())))]
    #[test_case(r#"$$".div""#, vec!(Token::new(SelectAll, ".div".into())))]
    fn test_token(input: &str, exp: Vec<Token>) {
        let mut l = Lexer::new(input);
        for tt in exp.iter() {
            let t = l.next_token();
            assert_eq!(t, *tt);
        }
    }
}
