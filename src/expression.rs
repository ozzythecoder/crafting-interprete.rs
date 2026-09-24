use std::{cell::RefCell, fmt::Display, rc::Rc};

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
    Variable(Variable),
    Assignment(Assignment),
    Call(Call),
}

/// The result of an evaluated expression. AKA an r-value
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

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Callable(_) => write!(f, "<function>"),
            Self::Literal(l) => write!(f, "{}", l),
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
    pub id: usize,
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

#[derive(Debug, Clone, PartialEq)]
pub struct Variable {
    pub id: usize,
    pub name: Token,
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
            Literal::Boolean(b) => *b,
            _ => true,
        }
    }
}

impl Display for Literal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Literal::String(s) => write!(f, "{}", s),
            Literal::Int(i) => write!(f, "{}", i),
            Literal::Float(fl) => write!(f, "{}", fl),
            Literal::Boolean(b) => {
                if *b {
                    write!(f, "true")
                } else {
                    write!(f, "false")
                }
            }
            Literal::True => write!(f, "true"),
            Literal::False => write!(f, "false"),
            Literal::Nil => write!(f, "nil"),
        }
    }
}
