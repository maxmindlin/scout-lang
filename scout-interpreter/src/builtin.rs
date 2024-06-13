use std::sync::Arc;

use fantoccini::elements::Element;
use futures::{future::BoxFuture, FutureExt, TryFutureExt};

use crate::{object::Object, EvalError, EvalResult, ScrapeResultsPtr};

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
    Results,
    Len,
    Input,
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
            "results" => Some(Results),
            "len" => Some(Len),
            "input" => Some(Input),
            _ => None,
        }
    }

    pub async fn apply(&self, results: ScrapeResultsPtr, args: Vec<Arc<Object>>) -> EvalResult {
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
                apply_elem_fn(&args[0], |elem| {
                    async move { Object::Str(elem.text().await.unwrap_or("".into())) }.boxed()
                })
                .await
            }
            Href => {
                assert_param_len!(args, 1);
                apply_elem_fn(&args[0], |elem| {
                    async move {
                        Object::Str(
                            elem.prop("href")
                                .await
                                .unwrap_or(Option::None)
                                .unwrap_or("".into()),
                        )
                    }
                    .boxed()
                })
                .await
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
            Results => {
                let json = results.lock().await.to_json();
                println!("{}", json);
                Ok(Arc::new(Object::Null))
            }
            Len => {
                assert_param_len!(args, 1);
                let len = match &*args[0] {
                    Object::List(v) => Ok(v.len() as f64),
                    Object::Str(s) => Ok(s.len() as f64),
                    _ => Err(EvalError::InvalidFnParams),
                }?;

                Ok(Arc::new(Object::Number(len)))
            }
            Input => {
                assert_param_len!(args, 2);
                match (&*args[0], &*args[1]) {
                    (Object::Node(elem), Object::Str(s)) => {
                        elem.send_keys(s)
                            .map_err(|_| EvalError::BrowserError)
                            .await?;
                        Ok(Arc::new(Object::Null))
                    }
                    _ => Err(EvalError::InvalidFnParams),
                }
            }
        }
    }
}

async fn apply_elem_fn(
    arg: &Object,
    f: impl Fn(&'_ Element) -> BoxFuture<'_, Object>,
) -> EvalResult {
    match arg {
        Object::Node(elem) => Ok(Arc::new(f(elem).await)),
        Object::List(list) => {
            let mut res = Vec::new();
            for obj in list.iter() {
                if let Object::Node(elem) = &*obj.clone() {
                    res.push(Arc::new(f(elem).await));
                } else {
                    return Err(EvalError::InvalidUsage);
                }
            }
            Ok(Arc::new(Object::List(res)))
        }
        _ => Err(EvalError::InvalidFnParams),
    }
}
