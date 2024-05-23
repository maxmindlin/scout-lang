use std::{collections::HashMap, fmt::Display};

use scout_parser::ast::Identifier;

pub enum Object {
    Null,
    Error,
    Map(HashMap<Identifier, Object>),
    Str(String),
    // @TODO: should contain the actual node, not a string
    Node(String),
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
                    write!(f, "{}: {}", i, o)?;
                }
                write!(f, "}}")
            }
            Str(s) => write!(f, "\"{}\"", s),
            Node(_) => write!(f, "Node"),
        }
    }
}
