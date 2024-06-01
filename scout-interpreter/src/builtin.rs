use crate::object::Object;

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
                if args.len() != 1 {
                    return Object::Error;
                }

                if let Object::Node(elem) = &args[0] {
                    Object::Str(elem.text().await.unwrap())
                } else {
                    Object::Error
                }
            }
            Href => {
                if args.len() != 1 {
                    return Object::Error;
                }

                if let Object::Node(elem) = &args[0] {
                    Object::Str(elem.prop("href").await.unwrap().unwrap_or("".into()))
                } else {
                    Object::Error
                }
            }
            Trim => {
                if args.len() != 1 {
                    return Object::Error;
                }

                if let Object::Str(s) = &args[0] {
                    Object::Str(s.trim().to_owned())
                } else {
                    Object::Error
                }
            }
        }
    }
}
