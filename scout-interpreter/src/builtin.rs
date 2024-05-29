use scraper::Html;

use crate::object::Object;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum BuiltinKind {
    Print,
    TextContent,
    Trim,
}

impl BuiltinKind {
    pub fn is_from(s: &str) -> Option<Self> {
        use BuiltinKind::*;
        match s {
            "print" => Some(Print),
            "textContent" => Some(TextContent),
            "trim" => Some(Trim),
            _ => None,
        }
    }

    pub fn apply(&self, args: Vec<Object>) -> Object {
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

                if let Object::Node(html) = &args[0] {
                    let doc = Html::parse_fragment(html);
                    let txt = doc.root_element().text();
                    Object::Str(txt.collect())
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
