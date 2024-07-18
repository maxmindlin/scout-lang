use std::{collections::HashMap, fmt::Display, sync::Arc};

use futures::{future::BoxFuture, lock::Mutex, FutureExt};
use scout_parser::ast::{Block, FnParam, Identifier};
use serde_json::{json, Value};

use crate::env::EnvPointer;

#[derive(Debug)]
pub enum Object {
    Null,
    Map(Mutex<HashMap<Identifier, Arc<Object>>>),
    Str(String),
    Node(fantoccini::elements::Element),
    List(Mutex<Vec<Arc<Object>>>),
    Boolean(bool),
    Number(f64),
    Fn(Vec<FnParam>, Block),
    Return(Arc<Object>),
    Module(EnvPointer),
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

    pub async fn into_iterable(&self) -> Option<impl IntoIterator<Item = Arc<Object>>> {
        use Object::*;
        match self {
            List(v) => {
                let inner = v.lock().await;

                Some(inner.clone())
            }
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

    pub fn to_display(&self) -> BoxFuture<String> {
        use Object::*;
        async move {
            match self {
                Null => "Null".into(),
                Map(hash) => {
                    let inner = hash.lock().await;
                    let mut out = "{ ".to_string();
                    for (idx, (i, o)) in inner.iter().enumerate() {
                        out.push_str(&format!("{}: {}", i, o));
                        if idx != inner.len() - 1 {
                            out.push_str(", ");
                        }
                    }
                    out.push_str(" }");
                    out
                }
                Str(s) => format!("\"{}\"", s),
                Node(_) => "Node".into(),
                List(mutex) => {
                    let inner = mutex.lock().await;

                    let mut out: String = "[".into();
                    for (i, obj) in inner.iter().enumerate() {
                        out.push_str(obj.to_display().await.as_str());
                        if i != inner.len() - 1 {
                            out.push_str(", ");
                        }
                    }
                    out.push(']');

                    out
                }
                Boolean(b) => format!("{}", b),
                Number(n) => format!("{}", n),
                _ => "object".into(),
            }
        }
        .boxed()
    }

    pub fn eq<'a>(&'a self, other: &'a Self) -> BoxFuture<'a, bool> {
        async move {
            use Object::*;
            match (self, other) {
                (Null, Null) => true,
                (Map(a), Map(b)) => {
                    let a_i = a.lock().await;
                    let b_i = b.lock().await;
                    for key in a_i.keys() {
                        match b_i.get(key) {
                            Some(obj) => {
                                if !a_i.get(key).unwrap().eq(obj).await {
                                    return false;
                                }
                            }
                            None => return false,
                        }
                    }
                    true
                }
                (Str(a), Str(b)) => a == b,
                // @TODO: check if this is even correct
                (Node(a), Node(b)) => a.element_id() == b.element_id(),
                (List(a), List(b)) => {
                    let a_i = a.lock().await;
                    let b_i = b.lock().await;

                    if a_i.len() != b_i.len() {
                        return false;
                    }

                    for idx in 0..(a_i.len() - 1) {
                        if !a_i[idx].eq(&b_i[idx]).await {
                            return false;
                        }
                    }

                    true
                }
                (Boolean(a), Boolean(b)) => a == b,
                (Number(a), Number(b)) => a == b,
                _ => false,
            }
        }
        .boxed()
    }

    pub async fn to_json(&self) -> Value {
        use Object::*;
        match self {
            Null => Value::Null,
            Str(s) => Value::String(s.to_owned()),
            // @TODO handle this better
            Node(_) => Value::String("Node".to_owned()),
            List(list) => vec_to_json(list).await,
            Map(map) => {
                let inner = map.lock().await;
                Value::Object(obj_map_to_json(&*inner).await)
            }
            Boolean(b) => Value::Bool(*b),
            Number(n) => json!(n),
            Fn(_, _) => panic!("cant serialize func"),
            _ => panic!("cant serialize object"),
        }
    }

    pub async fn is_truthy(&self) -> bool {
        use Object::*;
        match self {
            Null => false,
            Str(s) => !s.is_empty(),
            Map(m) => !m.lock().await.is_empty(),
            List(v) => !v.lock().await.is_empty(),
            Boolean(b) => *b,
            // @TODO: Idk what truthiness of floats should be
            Number(n) => *n > 0.0,
            _ => true,
        }
    }
}

impl Display for Object {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use Object::*;
        match self {
            Null => write!(f, "Null"),
            Str(s) => write!(f, "\"{}\"", s),
            Node(_) => write!(f, "Node"),
            List(_objs) => write!(f, "list"),
            Boolean(b) => write!(f, "{}", b),
            Number(n) => write!(f, "{}", n),
            _ => write!(f, "object"),
        }
    }
}

fn vec_to_json<'a>(v: &'a Mutex<Vec<Arc<Object>>>) -> BoxFuture<'a, Value> {
    async move {
        let mut out = Vec::new();
        let inner = v.lock().await;
        for obj in &*inner {
            out.push(obj.clone().to_json().await);
        }
        Value::Array(out)
    }
    .boxed()
}

pub fn obj_map_to_json(
    map: &HashMap<Identifier, Arc<Object>>,
) -> BoxFuture<serde_json::Map<String, Value>> {
    async move {
        let mut out = serde_json::Map::new();
        for (ident, obj) in map.iter() {
            out.insert(ident.name.clone(), obj.to_json().await);
        }
        out
    }
    .boxed()
}
