use crate::token::Token;

#[derive(Debug, Clone)]
pub enum Expr {
    Binary(Binary),
    Unary(Unary),
    Grouping(Grouping),
    Literal(Literal),
}

#[derive(Debug, Clone)]
pub struct Binary {
    pub left: Box<Expr>,
    pub operator: Token,
    pub right: Box<Expr>,
}

#[derive(Debug, Clone)]
pub struct Unary {
    pub operator: Token,
    pub right: Box<Expr>,
}

#[derive(Debug, Clone)]
pub struct Grouping {
    pub expression: Box<Expr>,
}

#[derive(Debug, Clone)]
pub enum Literal {
    String(String),
    Int(i32),
    Float(f32),
}

impl ToString for Literal {
    fn to_string(&self) -> String {
        match self {
            Literal::String(s) => s.clone(),
            Literal::Int(i) => i.to_string(),
            Literal::Float(f) => f.to_string(),
        }
    }
}

pub fn print(expr: &Expr) -> String {
    match expr {
        Expr::Binary(b) => parenthesize(&b.operator.lexeme, &[b.left.clone(), b.right.clone()]),
        Expr::Unary(u) => parenthesize(&u.operator.lexeme, &[u.right.clone()]),
        Expr::Grouping(g) => parenthesize("group", &[g.expression.clone()]),
        Expr::Literal(l) => l.to_string(),
    }
}

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
