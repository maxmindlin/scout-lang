use scout_parser::ast::HashLiteral;

pub enum Object {
    Null,
    Error,
    Map(HashLiteral),
}