use std::{collections::HashMap, fmt::Display, sync::Arc};

use scout_parser::ast::Identifier;
use serde_json::{json, Value};

#[derive(Debug)]
pub enum Object {
    Null,
    Map(HashMap<Identifier, Arc<Object>>),
    Str(String),
    Node(fantoccini::elements::Element),
    List(Vec<Arc<Object>>),
    Boolean(bool),
    Number(f64),
}

impl PartialEq for Object {
    fn eq(&self, other: &Self) -> bool {
        use Object::*;
        match (self, other) {
            (Null, Null) => true,
            (Map(a), Map(b)) => a == b,
            (Str(a), Str(b)) => a == b,
            // @TODO: check if this is even correct
            (Node(a), Node(b)) => a.element_id() == b.element_id(),
            (List(a), List(b)) => a == b,
            (Boolean(a), Boolean(b)) => a == b,
            (Number(a), Number(b)) => a == b,
            _ => false,
        }
    }
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
            List(objs) => write!(f, "[Object; {}]", objs.len()),
            Boolean(b) => write!(f, "{}", b),
            Number(n) => write!(f, "{}", n),
        }
    }
}

impl Object {
    pub fn to_json(&self) -> Value {
        use Object::*;
        match self {
            Null => Value::Null,
            Str(s) => Value::String(s.to_owned()),
            // @TODO handle this better
            Node(_) => Value::String("Node".to_owned()),
            List(list) => Value::Array(list.iter().map(|obj| obj.to_json()).collect()),
            Map(map) => Value::Object(obj_map_to_json(map)),
            Boolean(b) => Value::Bool(*b),
            Number(n) => json!(n),
        }
    }

    pub fn is_truthy(&self) -> bool {
        use Object::*;
        match self {
            Null => false,
            Str(s) => !s.is_empty(),
            Map(m) => !m.is_empty(),
            Node(_) => true,
            List(v) => !v.is_empty(),
            Boolean(b) => *b,
            // @TODO: Idk what truthiness of floats should be
            Number(n) => *n > 0.0,
        }
    }
}

pub fn obj_map_to_json(map: &HashMap<Identifier, Arc<Object>>) -> serde_json::Map<String, Value> {
    let mut out = serde_json::Map::new();
    for (ident, obj) in map.iter() {
        out.insert(ident.name.clone(), obj.to_json());
    }
    out
}
