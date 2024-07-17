use std::{collections::HashMap, env, sync::Arc, thread::sleep, time::Duration};

use fantoccini::{
    actions::{InputSource, KeyAction, KeyActions},
    client,
    cookies::Cookie,
    elements::Element,
    key::Key,
};
use futures::{future::BoxFuture, lock::Mutex, FutureExt, TryFutureExt};
use scout_parser::ast::Identifier;

use crate::{object::Object, EvalError, EvalResult, ScrapeResultsPtr};

macro_rules! assert_param_len {
    ($arg:expr, $len:expr) => {
        if $arg.len() < $len {
            return Err($crate::EvalError::InvalidFnParams);
        }
    };
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum BuiltinKind {
    Args,
    Print,
    TextContent,
    Href,
    Click,
    Results,
    Len,
    Input,
    Contains,
    Type,
    KeyPress,
    Number,
    Url,
    Sleep,
    IsWhitespace,
    List,
    Push,
    Cookies,
    SetCookies,
}

impl BuiltinKind {
    pub fn is_from(s: &str) -> Option<Self> {
        use BuiltinKind::*;
        match s {
            "isWhitespace" => Some(IsWhitespace),
            "url" => Some(Url),
            "number" => Some(Number),
            "args" => Some(Args),
            "print" => Some(Print),
            "textContent" => Some(TextContent),
            "href" => Some(Href),
            "click" => Some(Click),
            "results" => Some(Results),
            "len" => Some(Len),
            "input" => Some(Input),
            "contains" => Some(Contains),
            "type" => Some(Type),
            "keyAction" => Some(KeyPress),
            "sleep" => Some(Sleep),
            "list" => Some(List),
            "push" => Some(Push),
            "cookies" => Some(Cookies),
            "setCookies" => Some(SetCookies),
            _ => None,
        }
    }

    pub async fn apply(
        &self,
        crawler: &fantoccini::Client,
        results: ScrapeResultsPtr,
        args: Vec<Arc<Object>>,
    ) -> EvalResult {
        use BuiltinKind::*;
        match self {
            Cookies => {
                let cookies = crawler
                    .get_all_cookies()
                    .await?
                    .iter()
                    .map(|c| {
                        (
                            Identifier::new(c.name().to_string()),
                            Arc::new(Object::Str(c.value().to_string())),
                        )
                    })
                    .collect::<HashMap<Identifier, Arc<Object>>>();

                Ok(Arc::new(Object::Map(Mutex::new(cookies))))
            }
            SetCookies => {
                assert_param_len!(args, 1);
                if let Object::Map(m) = &*args[0] {
                    let inner = m.lock().await;
                    crawler.delete_all_cookies().await?;
                    for (key, val) in inner.iter() {
                        let cookie = Cookie::new(key.name.clone(), val.to_string());
                        crawler.add_cookie(cookie).await?;
                    }

                    Ok(Arc::new(Object::Null))
                } else {
                    Err(EvalError::InvalidFnParams)
                }
            }
            Push => {
                assert_param_len!(args, 2);
                match (&*args[0], args[1].clone()) {
                    (Object::List(l), o) => {
                        l.lock().await.push(o.clone());
                        Ok(Arc::new(Object::Null))
                    }
                    _ => Err(EvalError::InvalidFnParams),
                }
            }
            List => {
                assert_param_len!(args, 1);
                if let Some(iterable) = args[0].into_iterable().await {
                    Ok(Arc::new(Object::List(Mutex::new(
                        iterable.into_iter().collect(),
                    ))))
                } else {
                    Err(EvalError::InvalidFnParams)
                }
            }
            IsWhitespace => {
                assert_param_len!(args, 1);
                if let Object::Str(s) = &*args[0] {
                    let is_whitespace = s.chars().all(|c| c.is_whitespace());
                    Ok(Arc::new(Object::Boolean(is_whitespace)))
                } else {
                    Err(EvalError::InvalidFnParams)
                }
            }
            Sleep => {
                assert_param_len!(args, 1);
                if let Object::Number(ms) = &*args[0] {
                    sleep(Duration::from_millis(ms.round() as u64));
                    Ok(Arc::new(Object::Null))
                } else {
                    Err(EvalError::InvalidFnParams)
                }
            }
            Url => {
                let url = crawler.current_url().await?;
                Ok(Arc::new(Object::Str(url.to_string())))
            }
            Number => {
                assert_param_len!(args, 1);
                if let Object::Str(s) = &*args[0] {
                    match s.parse::<f64>() {
                        Ok(n) => Ok(Arc::new(Object::Number(n))),
                        Err(_) => Err(EvalError::InvalidUsage(
                            "input to number() must be a valid number".into(),
                        )),
                    }
                } else {
                    Err(EvalError::InvalidUsage(
                        "number() takes a str as input".to_owned(),
                    ))
                }
            }
            Args => {
                let env_args = env::args().collect::<Vec<String>>();
                let mut out = Vec::new();
                // start at 1 because idx 0 is the executable location
                for idx in 1..env_args.len() {
                    out.push(Arc::new(Object::Str(env_args[idx].clone())));
                }
                Ok(Arc::new(Object::List(Mutex::new(out))))
            }
            Type => {
                assert_param_len!(args, 1);
                Ok(Arc::new(Object::Str(args[0].type_str().to_string())))
            }
            Print => {
                for obj in args {
                    let display = obj.to_display().await;
                    println!("{display}");
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
            Results => {
                let json = results.lock().await.to_json();
                println!("{}", json);
                Ok(Arc::new(Object::Null))
            }
            Len => {
                assert_param_len!(args, 1);
                let len = match &*args[0] {
                    Object::List(v) => Ok(v.lock().await.len() as f64),
                    Object::Str(s) => Ok(s.len() as f64),
                    _ => Err(EvalError::InvalidFnParams),
                }?;

                Ok(Arc::new(Object::Number(len)))
            }
            KeyPress => {
                assert_param_len!(args, 1);
                if let Object::Str(code_str) = &*args[0] {
                    let code = code_str.chars().next().ok_or(EvalError::InvalidFnParams)?;
                    let actions = KeyActions::new("keypress".to_owned())
                        .then(KeyAction::Down { value: code });

                    crawler.perform_actions(actions).await?;
                    Ok(Arc::new(Object::Null))
                } else {
                    Err(EvalError::InvalidFnParams)
                }
            }
            Input => {
                assert_param_len!(args, 2);
                match (&*args[0], &*args[1]) {
                    (Object::Node(elem), Object::Str(s)) => {
                        elem.send_keys(s).map_err(EvalError::BrowserError).await?;

                        if args.len() > 2 && args[2].is_truthy().await {
                            let actions =
                                KeyActions::new("enter".to_owned()).then(KeyAction::Down {
                                    value: Key::Return.into(),
                                });
                            crawler.perform_actions(actions).await?;
                        }
                        Ok(Arc::new(Object::Null))
                    }
                    _ => Err(EvalError::InvalidFnParams),
                }
            }
            Contains => {
                assert_param_len!(args, 2);
                match &*args[0] {
                    Object::Str(s) => match &*args[1] {
                        Object::Str(sub) => {
                            let contains = s.contains(sub);
                            Ok(Arc::new(Object::Boolean(contains)))
                        }
                        _ => Err(EvalError::InvalidFnParams),
                    },
                    Object::List(v) => {
                        let inner = v.lock().await;
                        let mut contains = false;
                        for obj in inner.iter() {
                            if obj.eq(&args[1]).await {
                                contains = true;
                                break;
                            }
                        }
                        Ok(Arc::new(Object::Boolean(contains)))
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
            let inner = list.lock().await;
            for obj in inner.iter() {
                if let Object::Node(elem) = &*obj.clone() {
                    res.push(Arc::new(f(elem).await));
                } else {
                    return Err(EvalError::InvalidUsage(
                        "cannot run builtin node fn against non-node".into(),
                    ));
                }
            }
            Ok(Arc::new(Object::List(Mutex::new(res))))
        }
        _ => Err(EvalError::InvalidFnParams),
    }
}
