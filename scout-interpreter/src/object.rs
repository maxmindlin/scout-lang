use std::{collections::HashMap, fmt::Display};

use scout_parser::ast::Identifier;

#[derive(Debug)]
pub enum Object {
    Null,
    Error,
    Map(HashMap<Identifier, Object>),
    Str(String),
    Node(fantoccini::elements::Element),
}

impl Object {
    pub fn is_error(&self) -> bool {
        match self {
            Self::Error => true,
            _ => false,
        }
    }
}

impl Display for Object {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use Object::*;
        match self {
            Null => write!(f, "Null"),
            Error => write!(f, "Error"),
            Map(hash) => {
                write!(f, "{{")?;
                for (i, o) in hash.iter() {
                    write!(f, "{}: {} ", i, o)?;
                }
                write!(f, "}}")
            }
            Str(s) => write!(f, "\"{}\"", s),
            Node(s) => write!(f, "Node"),
        }
    }
}
