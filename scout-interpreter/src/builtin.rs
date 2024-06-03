use std::sync::Arc;

use crate::{object::Object, EvalError, EvalResult};

macro_rules! assert_param_len {
    ($arg:expr, $len:expr) => {
        if $arg.len() != $len {
            return Err($crate::EvalError::InvalidFnParams);
        }
    };
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum BuiltinKind {
    Print,
    TextContent,
    Href,
    Trim,
    Click,
}

impl BuiltinKind {
    pub fn is_from(s: &str) -> Option<Self> {
        use BuiltinKind::*;
        match s {
            "print" => Some(Print),
            "textContent" => Some(TextContent),
            "trim" => Some(Trim),
            "href" => Some(Href),
            "click" => Some(Click),
            _ => None,
        }
    }

    pub async fn apply(&self, args: Vec<Arc<Object>>) -> EvalResult {
        use BuiltinKind::*;
        match self {
            Print => {
                for obj in args {
                    println!("{obj}");
                }
                Ok(Arc::new(Object::Null))
            }
            TextContent => {
                assert_param_len!(args, 1);
                if let Object::Node(elem) = &*args[0] {
                    Ok(Arc::new(Object::Str(elem.text().await.unwrap())))
                } else {
                    Err(EvalError::InvalidFnParams)
                }
            }
            Href => {
                assert_param_len!(args, 1);
                if let Object::Node(elem) = &*args[0] {
                    Ok(Arc::new(Object::Str(
                        elem.prop("href").await.unwrap().unwrap_or("".into()),
                    )))
                } else {
                    Err(EvalError::InvalidFnParams)
                }
            }
            Click => {
                assert_param_len!(args, 1);
                if let Object::Node(elem) = &*args[0] {
                    let _ = elem.click().await;
                    Ok(Arc::new(Object::Null))
                } else {
                    Err(EvalError::InvalidFnParams)
                }
            }
            Trim => {
                assert_param_len!(args, 1);
                if let Object::Str(s) = &*args[0] {
                    Ok(Arc::new(Object::Str(s.trim().to_owned())))
                } else {
                    Err(EvalError::InvalidFnParams)
                }
            }
        }
    }
}
