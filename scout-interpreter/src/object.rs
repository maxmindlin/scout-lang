use std::{collections::HashMap, fmt::Display, sync::Arc};

use scout_parser::ast::{Block, Identifier};
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
    Fn(Vec<Identifier>, Block),
    Return(Arc<Object>),
}

impl Object {
    pub fn type_str(&self) -> &str {
        use Object::*;
        match self {
            Null => "null",
            Map(_) => "map",
            Str(_) => "string",
            Node(_) => "node",
            List(_) => "list",
            Boolean(_) => "bool",
            Number(_) => "number",
            Fn(_, _) => "fn",
            _ => "object",
        }
    }

    pub fn into_iterable(&self) -> Option<impl IntoIterator<Item = Arc<Object>>> {
        use Object::*;
        match self {
            List(v) => Some(v.clone()),
            Str(s) => {
                let new_vec: Vec<Arc<Object>> = s
                    .chars()
                    .map(|c| Arc::new(Object::Str(c.to_string())))
                    .collect();

                Some(new_vec)
            }
            _ => None,
        }
    }
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
            List(objs) => {
                write!(f, "[")?;
                for (i, obj) in objs.iter().enumerate() {
                    if i == objs.len() - 1 {
                        write!(f, "{}", obj)?;
                    } else {
                        write!(f, "{}, ", obj)?;
                    }
                }
                write!(f, "]")
            }
            Boolean(b) => write!(f, "{}", b),
            Number(n) => write!(f, "{}", n),
            _ => write!(f, "object"),
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
            Fn(_, _) => panic!("cant serialize func"),
            _ => panic!("cant serialize object"),
        }
    }

    pub fn is_truthy(&self) -> bool {
        use Object::*;
        match self {
            Null => false,
            Str(s) => !s.is_empty(),
            Map(m) => !m.is_empty(),
            List(v) => !v.is_empty(),
            Boolean(b) => *b,
            // @TODO: Idk what truthiness of floats should be
            Number(n) => *n > 0.0,
            _ => true,
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
