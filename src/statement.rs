use crate::{expression::Expr, token::Token};

#[derive(Debug, Clone)]
pub enum Stmt {
    Expression(Expr),
    Print(Expr),
    Var { name: Token, initializer: Option<Expr> },
    Block(Vec<Stmt>),
    If { condition: Expr, then_branch: Box<Stmt>, else_branch: Option<Box<Stmt>> }
}
