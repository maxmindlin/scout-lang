#[derive(Debug, Clone, PartialEq, Eq, Copy)]
pub enum TokenKind {
    Illegal,
    EOF,
    Ident,
    Int,
    Float,
    Str,

    Comma,
    Colon,
    Pipe,
    LParen,
    RParen,
    LBrace,
    RBrace,
    Select,
    SelectAll,

    // Keywords
    For,
    In,
    Do,
    End,
    Goto,
    Scrape,
}

impl TokenKind {
    pub fn is_to_keyword(literal: &str) -> Option<Self> {
        use TokenKind::*;
        match literal {
            "for" => Some(For),
            "in" => Some(In),
            "do" => Some(Do),
            "end" => Some(End),
            "goto" => Some(Goto),
            "scrape" => Some(Scrape),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Token {
    pub kind: TokenKind,
    pub literal: String,
}

impl Token {
    pub fn new(kind: TokenKind, literal: String) -> Self {
        Self { kind, literal }
    }
}
