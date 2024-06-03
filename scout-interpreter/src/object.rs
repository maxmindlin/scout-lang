use std::{collections::HashMap, fmt::Display, sync::Arc};

use scout_parser::ast::Identifier;

#[derive(Debug)]
pub enum Object {
    Null,
    Map(HashMap<Identifier, Arc<Object>>),
    Str(String),
    Node(fantoccini::elements::Element),
    List(Vec<Arc<Object>>),
}

impl Display for Object {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use Object::*;
        match self {
            Null => write!(f, "Null"),
            Map(hash) => {
                write!(f, "{{")?;
                for (i, o) in hash.iter() {
                    write!(f, "{}: {} ", i, o)?;
                }
                write!(f, "}}")
            }
            Str(s) => write!(f, "\"{}\"", s),
            Node(_) => write!(f, "Node"),
            List(objs) => write!(f, "[Node; {}]", objs.len()),
        }
    }
}
