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
    Equal,

    // Keywords
    If,
    For,
    In,
    Do,
    End,
    Goto,
    Scrape,
    Screenshot,
}

impl TokenKind {
    pub fn is_to_keyword(literal: &str) -> Option<Self> {
        use TokenKind::*;
        match literal {
            "for" => Some(For),
            "in" => Some(In),
            "if" => Some(If),
            "do" => Some(Do),
            "end" => Some(End),
            "goto" => Some(Goto),
            "scrape" => Some(Scrape),
            "screenshot" => Some(Screenshot),
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
