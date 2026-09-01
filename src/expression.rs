use crate::token::Token;

/// Defines all expression types, as defined in [the grammar definition](./lox_grammar.txt).
#[derive(Debug, Clone)]
pub enum Expr {
    Binary(Binary),
    Unary(Unary),
    Grouping(Grouping),
    Literal(Literal),
}

/// An expression that compares or operates on two expressions.
/// Examples: `(1 * 2)`, `(3 < 4)`
#[derive(Debug, Clone)]
pub struct Binary {
    pub left: Box<Expr>,
    pub operator: Token,
    pub right: Box<Expr>,
}

/// An expression that operates on a single expression.
/// Example: `(- 1)`
#[derive(Debug, Clone)]
pub struct Unary {
    pub operator: Token,
    pub right: Box<Expr>,
}

/// An arbitrary expression grouping.
#[derive(Debug, Clone)]
pub struct Grouping {
    pub expression: Box<Expr>,
}

/// A literal value. Can be a string, 32-bit integer, or 32-bit float.
#[derive(Debug, Clone, PartialEq)]
pub enum Literal {
    String(String),
    Int(i32),
    Float(f32),
    Boolean(bool),
    True,
    False,
    Nil,
}

impl ToString for Literal {
    fn to_string(&self) -> String {
        match self {
            Literal::String(s) => s.clone(),
            Literal::Int(i) => i.to_string(),
            Literal::Float(f) => f.to_string(),
            Literal::Boolean(b) => {
                if *b {
                    "true".to_owned()
                } else {
                    "false".to_owned()
                }
            }
            Literal::True => "true".to_owned(),
            Literal::False => "false".to_owned(),
            Literal::Nil => "nil".to_owned(),
        }
    }
}
