use std::fmt::{Display, write};

use crate::token::{Token, TokenType};

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

#[derive(Debug)]
pub struct Value(Literal);

impl ToString for Value {
    fn to_string(&self) -> String {
        self.0.to_string()
    }
}

#[derive(Debug)]
pub struct RuntimeError {
    pub token: Token,
    pub message: String,
}

pub fn interpret(expr: Expr) -> Result<Value, RuntimeError> {
    evaluate(expr)
}

pub fn evaluate(expr: Expr) -> Result<Value, RuntimeError> {
    match expr {
        Expr::Literal(l) => Ok(Value(l)),
        Expr::Grouping(g) => evaluate(*g.expression),
        Expr::Unary(u) => {
            let right = evaluate(*u.right)?;

            match u.operator.token_type {
                // (-1)
                TokenType::Minus => match right.0 {
                    Literal::Int(int) => Ok(Value(Literal::Int(-1 * int))),
                    Literal::Float(float) => Ok(Value(Literal::Float(-1.0 * float))),
                    _ => Err(RuntimeError {
                        token: u.operator.clone(),
                        message: "Cannot negate a non-number".to_owned(),
                    }),
                },
                // (!false)
                TokenType::Bang => match right.0 {
                    Literal::False => Ok(Value(Literal::True)),
                    Literal::Nil => Ok(Value(Literal::True)),
                    _ => Ok(Value(Literal::False)),
                },
                _ => Err(RuntimeError {
                    token: u.operator.clone(),
                    message: "Invalid unary operator".to_owned(),
                }),
            }
        }
        Expr::Binary(b) => {
            let left = evaluate(*b.left)?;
            let right = evaluate(*b.right)?;

            match b.operator.token_type {
                TokenType::Minus => match (left.0, right.0) {
                    (Literal::Float(l), Literal::Float(r)) => Ok(Value(Literal::Float(l - r))),
                    (Literal::Int(l), Literal::Int(r)) => Ok(Value(Literal::Int(l - r))),
                    (Literal::Float(l), Literal::Int(r)) => Ok(Value(Literal::Float(l - r as f32))),
                    (Literal::Int(l), Literal::Float(r)) => Ok(Value(Literal::Float(l as f32 - r))),
                    _ => Err(RuntimeError {
                        token: b.operator.clone(),
                        message: "Invalid subtraction operation".to_owned(),
                    }),
                },
                TokenType::Slash => match (left.0, right.0) {
                    (Literal::Float(l), Literal::Float(r)) => Ok(Value(Literal::Float(l / r))),
                    (Literal::Int(l), Literal::Int(r)) => Ok(Value(Literal::Int(l / r))),
                    (Literal::Float(l), Literal::Int(r)) => Ok(Value(Literal::Float(l / r as f32))),
                    (Literal::Int(l), Literal::Float(r)) => Ok(Value(Literal::Float(l as f32 / r))),
                    _ => Err(RuntimeError {
                        token: b.operator.clone(),
                        message: "Invalid division operation".to_owned(),
                    }),
                },
                TokenType::Star => match (left.0, right.0) {
                    (Literal::Float(l), Literal::Float(r)) => Ok(Value(Literal::Float(l * r))),
                    (Literal::Int(l), Literal::Int(r)) => Ok(Value(Literal::Int(l * r))),
                    (Literal::Float(l), Literal::Int(r)) => Ok(Value(Literal::Float(l * r as f32))),
                    (Literal::Int(l), Literal::Float(r)) => Ok(Value(Literal::Float(l as f32 * r))),
                    _ => Err(RuntimeError {
                        token: b.operator.clone(),
                        message: "Invalid multiplication operation".to_owned(),
                    }),
                },
                TokenType::Plus => match (left.0, right.0) {
                    (Literal::Float(l), Literal::Float(r)) => Ok(Value(Literal::Float(l + r))),
                    (Literal::Int(l), Literal::Int(r)) => Ok(Value(Literal::Int(l + r))),
                    (Literal::Float(l), Literal::Int(r)) => Ok(Value(Literal::Float(l + r as f32))),
                    (Literal::Int(l), Literal::Float(r)) => Ok(Value(Literal::Float(l as f32 + r))),
                    // string concatenation!
                    (Literal::String(l), Literal::String(r)) => Ok(Value(Literal::String(l + &r))),
                    _ => Err(RuntimeError {
                        token: b.operator.clone(),
                        message: "Invalid addition operation".to_owned(),
                    }),
                },
                TokenType::Greater => match (left.0, right.0) {
                    (Literal::Float(l), Literal::Float(r)) => Ok(Value(Literal::Boolean(l > r))),
                    (Literal::Int(l), Literal::Int(r)) => Ok(Value(Literal::Boolean(l > r))),
                    (Literal::Float(l), Literal::Int(r)) => {
                        Ok(Value(Literal::Boolean(l > r as f32)))
                    }
                    (Literal::Int(l), Literal::Float(r)) => {
                        Ok(Value(Literal::Boolean(l as f32 > r)))
                    }
                    _ => Err(RuntimeError {
                        token: b.operator.clone(),
                        message: "Invalid comparison".to_owned(),
                    }),
                },
                TokenType::GreaterEqual => match (left.0, right.0) {
                    (Literal::Float(l), Literal::Float(r)) => Ok(Value(Literal::Boolean(l >= r))),
                    (Literal::Int(l), Literal::Int(r)) => Ok(Value(Literal::Boolean(l >= r))),
                    (Literal::Float(l), Literal::Int(r)) => {
                        Ok(Value(Literal::Boolean(l >= r as f32)))
                    }
                    (Literal::Int(l), Literal::Float(r)) => {
                        Ok(Value(Literal::Boolean(l as f32 >= r)))
                    }
                    _ => Err(RuntimeError {
                        token: b.operator.clone(),
                        message: "Invalid comparison".to_owned(),
                    }),
                },
                TokenType::Less => match (left.0, right.0) {
                    (Literal::Float(l), Literal::Float(r)) => Ok(Value(Literal::Boolean(l < r))),
                    (Literal::Int(l), Literal::Int(r)) => Ok(Value(Literal::Boolean(l < r))),
                    (Literal::Float(l), Literal::Int(r)) => {
                        Ok(Value(Literal::Boolean(l < r as f32)))
                    }
                    (Literal::Int(l), Literal::Float(r)) => {
                        Ok(Value(Literal::Boolean((l as f32) < r)))
                    }
                    _ => Err(RuntimeError {
                        token: b.operator.clone(),
                        message: "Invalid comparison".to_owned(),
                    }),
                },
                TokenType::LessEqual => match (left.0, right.0) {
                    (Literal::Float(l), Literal::Float(r)) => Ok(Value(Literal::Boolean(l <= r))),
                    (Literal::Int(l), Literal::Int(r)) => Ok(Value(Literal::Boolean(l <= r))),
                    (Literal::Float(l), Literal::Int(r)) => {
                        Ok(Value(Literal::Boolean(l <= r as f32)))
                    }
                    (Literal::Int(l), Literal::Float(r)) => {
                        Ok(Value(Literal::Boolean(l as f32 <= r)))
                    }
                    _ => Err(RuntimeError {
                        token: b.operator.clone(),
                        message: "Invalid comparison".to_owned(),
                    }),
                },
                TokenType::BangEqual => Ok(Value(Literal::Boolean(left.0 != right.0))),
                TokenType::EqualEqual => Ok(Value(Literal::Boolean(left.0 == right.0))),
                _ => Err(RuntimeError {
                    token: b.operator.clone(),
                    message: "Invalid binary operation".to_owned(),
                }),
            }
        }
    }
}

// The book implements a visitor pattern and instructs on writing a codegen tool to declare
// each node on the AST. Rust's enum types allow us to bypass this step completely.
pub fn print(expr: &Expr) -> String {
    match expr {
        Expr::Binary(b) => parenthesize(&b.operator.lexeme, &[b.left.clone(), b.right.clone()]),
        Expr::Unary(u) => parenthesize(&u.operator.lexeme, &[u.right.clone()]),
        Expr::Grouping(g) => parenthesize("group", &[g.expression.clone()]),
        Expr::Literal(l) => l.to_string(),
    }
}

/// Surround one or more expressions in parentheses.
fn parenthesize(lexeme: &str, exprs: &[Box<Expr>]) -> String {
    let mut statement = String::from("(");
    statement.push_str(lexeme);
    for expr in exprs {
        statement.push_str(" ");
        statement.push_str(&print(expr));
    }
    statement.push_str(")");
    statement
}
