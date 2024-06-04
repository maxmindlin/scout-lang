use std::{collections::HashMap, fmt::Display, sync::Arc};

use scout_parser::ast::Identifier;
use serde_json::Value;

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
