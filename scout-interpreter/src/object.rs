use std::collections::HashMap;

use scout_parser::ast::Identifier;
use scraper::ElementRef;

pub enum Object<'a> {
    Null,
    Error,
    Map(HashMap<Identifier, Object<'a>>),
    Str(String),
    Node(ElementRef<'a>),
}

