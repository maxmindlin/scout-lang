use std::sync::Arc;

use crate::object::Object;

macro_rules! assert_param_len {
    ($arg:expr, $len:expr) => {
        if $arg.len() != $len {
            return Arc::new($crate::object::Object::Error);
        }
    };
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum BuiltinKind {
    Print,
    TextContent,
    Href,
    Trim,
}

impl BuiltinKind {
    pub fn is_from(s: &str) -> Option<Self> {
        use BuiltinKind::*;
        match s {
            "print" => Some(Print),
            "textContent" => Some(TextContent),
            "trim" => Some(Trim),
            "href" => Some(Href),
            _ => None,
        }
    }

    pub async fn apply(&self, args: Vec<Arc<Object>>) -> Arc<Object> {
        use BuiltinKind::*;
        let out = match self {
            Print => {
                for obj in args {
                    println!("{obj}");
                }
                Object::Null
            }
            TextContent => {
                assert_param_len!(args, 1);
                if let Object::Node(elem) = &*args[0] {
                    Object::Str(elem.text().await.unwrap())
                } else {
                    Object::Error
                }
            }
            Href => {
                assert_param_len!(args, 1);
                if let Object::Node(elem) = &*args[0] {
                    Object::Str(elem.prop("href").await.unwrap().unwrap_or("".into()))
                } else {
                    Object::Error
                }
            }
            Trim => {
                assert_param_len!(args, 1);
                if let Object::Str(s) = &*args[0] {
                    Object::Str(s.trim().to_owned())
                } else {
                    Object::Error
                }
            }
        };
        Arc::new(out)
    }
}
