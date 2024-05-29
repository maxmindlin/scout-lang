use crate::object::Object;

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

    pub fn apply(&self, args: Vec<Object>) -> Object {
        use BuiltinKind::*;
        match self {
            Print => {
                println!("{self:?}");
                Object::Null
            }
            TextContent => {
                println!("{args:?}");
                Object::Str("textContent".to_string())
            }
        }
    }
}
