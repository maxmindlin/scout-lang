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
    LBracket,
    RBracket,
    Select,
    SelectAll,
    Assign,
    EQ,
    NEQ,
    Plus,
    Minus,
    Asterisk,
    Slash,
    GT,
    LT,
    GTE,
    LTE,
    Bang,
    DbColon,

    // Keywords
    If,
    Elif,
    Else,
    For,
    In,
    Do,
    End,
    Goto,
    Scrape,
    Screenshot,
    True,
    False,
    Def,
    Null,
    Return,
    Use,
    Try,
    Catch,
    Throw,
    Crawl,
    Where,
    And,
    Or,
}

impl TokenKind {
    pub fn is_to_keyword(literal: &str) -> Option<Self> {
        use TokenKind::*;
        match literal {
            "where" => Some(Where),
            "for" => Some(For),
            "in" => Some(In),
            "if" => Some(If),
            "elif" => Some(Elif),
            "else" => Some(Else),
            "do" => Some(Do),
            "end" => Some(End),
            "goto" => Some(Goto),
            "scrape" => Some(Scrape),
            "screenshot" => Some(Screenshot),
            "true" => Some(True),
            "false" => Some(False),
            "def" => Some(Def),
            "null" => Some(Null),
            "return" => Some(Return),
            "use" => Some(Use),
            "try" => Some(Try),
            "catch" => Some(Catch),
            "crawl" => Some(Crawl),
            "throw" => Some(Throw),
            "and" => Some(And),
            "or" => Some(Or),
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
