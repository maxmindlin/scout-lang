#[derive(Debug, PartialEq, Eq, Clone)]
pub enum BuiltinKind {
    Print,
    TextContent,
}

impl BuiltinKind {
    pub fn is_from(s: &str) -> Option<Self> {
        use BuiltinKind::*;
        match s {
            "print" => Some(Print),
            "textContent" => Some(TextContent),
            _ => None,
        }
    }
}
