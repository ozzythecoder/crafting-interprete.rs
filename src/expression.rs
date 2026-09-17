use std::{cell::RefCell, rc::Rc};

use crate::{
    interpreter::{Interpreter, Interrupt, RuntimeError}, statement::Stmt, token::Token,
};

pub trait IsTruthy {
    fn is_truthy(&self) -> bool;
}

/// Defines all expression types, as defined in [the grammar definition](./lox_grammar.txt).
#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Binary(Binary),
    Unary(Unary),
    Logical(Logical),
    Grouping(Grouping),
    Literal(Literal),
    Variable(Token),
    Assignment(Assignment),
    Call(Call),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Literal(Literal),
    Callable(Rc<RefCell<Callable>>),
}

impl IsTruthy for Value {
    fn is_truthy(&self) -> bool {
        match self {
            Self::Callable(_) => true,
            Self::Literal(l) => match l {
                Literal::False | Literal::Nil => false,
                Literal::Boolean(b) => *b,
                _ => true,
            },
        }
    }
}

impl ToString for Value {
    fn to_string(&self) -> String {
        match self {
            Self::Callable(_) => String::from("<native fn>"),
            Self::Literal(l) => l.to_string(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Call {
    pub callee: Box<Expr>,
    pub paren: Token,
    pub args: Vec<Expr>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Assignment {
    pub name: Token,
    pub value: Box<Expr>,
}

/// An expression that compares or operates on two expressions.
/// Examples: `(1 * 2)`, `(3 < 4)`
#[derive(Debug, Clone, PartialEq)]
pub struct Binary {
    pub left: Box<Expr>,
    pub operator: Token,
    pub right: Box<Expr>,
}

/// An expression that operates on a single expression.
/// Example: `(- 1)`
#[derive(Debug, Clone, PartialEq)]
pub struct Unary {
    pub operator: Token,
    pub right: Box<Expr>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Logical {
    pub left: Box<Expr>,
    pub operator: Token,
    pub right: Box<Expr>,
}

/// An arbitrary expression grouping.
#[derive(Debug, Clone, PartialEq)]
pub struct Grouping {
    pub expression: Box<Expr>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Callable {
    Function(Function),
    Native(NativeFunction),
    Class(Class),
}

pub fn to_callable_value(callable: Callable) -> Value {
    Value::Callable(Rc::new(RefCell::new(callable)))
}

#[derive(Debug, Clone, PartialEq)]
pub struct NativeFunction {
    pub name: String,
    pub arity: usize,
    pub func: fn(&mut Interpreter, Vec<Value>) -> Result<Value, Interrupt<RuntimeError>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Function {
    pub params: Vec<Token>,
    pub body: Vec<Stmt>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Class {
    pub args: Vec<Value>,
}

pub trait TCallable {
    fn arity(&self) -> usize;
}

impl TCallable for Function {
    fn arity(&self) -> usize {
        self.params.len()
    }
}

/// A literal value. Can be a string, 32-bit integer, 32-bit float, boolean, or nil.
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

impl IsTruthy for Callable {
    fn is_truthy(&self) -> bool {
        true
    }
}

impl IsTruthy for Literal {
    fn is_truthy(&self) -> bool {
        match self {
            Literal::False | Literal::Nil => false,
            Literal::Boolean(b) => b.clone(),
            _ => true,
        }
    }
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
