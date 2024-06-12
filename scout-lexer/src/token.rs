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
    Assign,
    EQ,
    NEQ,

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

    pub fn is_infix(&self) -> bool {
        use TokenKind::*;
        match self {
            EQ => true,
            NEQ => true,
            _ => false,
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
