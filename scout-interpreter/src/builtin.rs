use crate::object::Object;

macro_rules! assert_param_len {
    ($arg:expr, $len:expr) => {
        if $arg != $len {
            return $crate::object::Object::Null;
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

    pub async fn apply(&self, args: Vec<Object>) -> Object {
        use BuiltinKind::*;
        match self {
            Print => {
                for obj in args {
                    println!("{obj}");
                }
                Object::Null
            }
            TextContent => {
                assert_param_len!(args.len(), 1);
                if let Object::Node(elem) = &args[0] {
                    Object::Str(elem.text().await.unwrap())
                } else {
                    Object::Error
                }
            }
            Href => {
                assert_param_len!(args.len(), 1);
                if let Object::Node(elem) = &args[0] {
                    Object::Str(elem.prop("href").await.unwrap().unwrap_or("".into()))
                } else {
                    Object::Error
                }
            }
            Trim => {
                assert_param_len!(args.len(), 1);
                if let Object::Str(s) = &args[0] {
                    Object::Str(s.trim().to_owned())
                } else {
                    Object::Error
                }
            }
        }
    }
}
