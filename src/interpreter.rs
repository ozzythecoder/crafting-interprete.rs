use crate::{
    environment::Environment,
    expression::{Expr, Literal},
    statement::Stmt,
    token::{Token, TokenType},
};

#[derive(Debug)]
pub struct Value(Literal);

impl ToString for Value {
    fn to_string(&self) -> String {
        self.0.to_string()
    }
}

impl Value {
    fn is_truthy(&self) -> bool {
        match self.0 {
            Literal::False | Literal::Nil => false,
            Literal::Boolean(b) => b,
            _ => true,
        }
    }
}

#[derive(Debug)]
pub struct RuntimeError {
    pub token: Token,
    pub message: String,
}

pub struct Interpreter {
    environment: Environment,
}

impl Interpreter {
    pub fn new() -> Self {
        Interpreter {
            environment: Environment::new(None),
        }
    }

    pub fn interpret(&mut self, statements: &[Stmt]) -> Result<(), RuntimeError> {
        for stmt in statements {
            match self.evaluate(&stmt) {
                Some(r) => match r {
                    Ok(_) => continue,
                    Err(e) => return Err(e),
                },
                None => continue,
            };
        }
        Ok(())
    }

    fn evaluate(&mut self, stmt: &Stmt) -> Option<Result<Value, RuntimeError>> {
        match stmt {
            Stmt::Expression(e) => Some(self.evaluate_expression(e)),
            Stmt::Print(p) => match self.evaluate_expression(p) {
                Ok(val) => {
                    println!("{}", val.to_string());
                    None
                }
                Err(e) => Some(Err(e)),
            },
            Stmt::Var { name, initializer } => match initializer {
                Some(init) => match self.evaluate_expression(init) {
                    Ok(val) => {
                        self.environment.define(name, val.0);
                        None
                    }
                    Err(e) => Some(Err(e)),
                },
                None => {
                    self.environment.define(name, Literal::Nil);
                    None
                }
            },
            Stmt::Block(block) => {
                // create new environment, with current environment as enclosing
                let prev_env = std::mem::take(&mut self.environment);
                self.environment = Environment::new(Some(prev_env));

                // evaluate contents of block
                let result = self.interpret(block);

                // take enclosing environment back from child and reset
                let child_env = std::mem::take(&mut self.environment);
                self.environment = *child_env
                    .enclosing
                    .expect("Block env cannot be built without a parent");

                match result {
                    Ok(()) => None,
                    Err(e) => Some(Err(e)),
                }
            }
            Stmt::If {
                condition,
                then_branch,
                else_branch,
            } => match self.evaluate_expression(condition) {
                Ok(r) => {
                    if r.is_truthy() {
                        self.evaluate(then_branch)
                    } else if let Some(else_branch) = else_branch {
                        self.evaluate(else_branch)
                    } else {
                        None
                    }
                }
                Err(e) => Some(Err(e)),
            },
        }
    }

    fn evaluate_expression(&mut self, expr: &Expr) -> Result<Value, RuntimeError> {
        match expr {
            Expr::Literal(l) => Ok(Value(l.clone())),
            Expr::Grouping(g) => self.evaluate_expression(&g.expression),
            Expr::Variable(v) => Ok(Value(self.environment.get(v)?.clone())),
            Expr::Assignment(a) => {
                let val = self.evaluate_expression(&a.value)?;
                if let Err(e) = self.environment.assign(&a.name, &val.0) {
                    Err(e)
                } else {
                    Ok(val)
                }
            }
            Expr::Logical(l) => {
                let left = self.evaluate_expression(&l.left)?;

                if l.operator.token_type == TokenType::Or && left.is_truthy() {
                    Ok(left)
                } else if !left.is_truthy() {
                    Ok(left)
                } else {
                    self.evaluate_expression(&l.right)
                }
            }
            Expr::Unary(u) => {
                let right = self.evaluate_expression(&u.right)?;

                match u.operator.token_type {
                    // e.g. (-1)
                    TokenType::Minus => match right.0 {
                        Literal::Int(int) => Ok(Value(Literal::Int(-1 * int))),
                        Literal::Float(float) => Ok(Value(Literal::Float(-1.0 * float))),
                        _ => Err(RuntimeError {
                            token: u.operator.clone(),
                            message: "Cannot negate a non-number".to_owned(),
                        }),
                    },
                    // e.g. (!false), (-val)
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
                let left = self.evaluate_expression(&b.left)?;
                let right = self.evaluate_expression(&b.right)?;

                match b.operator.token_type {
                    TokenType::Minus => match (left.0, right.0) {
                        (Literal::Float(l), Literal::Float(r)) => Ok(Value(Literal::Float(l - r))),
                        (Literal::Int(l), Literal::Int(r)) => Ok(Value(Literal::Int(l - r))),
                        (Literal::Float(l), Literal::Int(r)) => {
                            Ok(Value(Literal::Float(l - r as f32)))
                        }
                        (Literal::Int(l), Literal::Float(r)) => {
                            Ok(Value(Literal::Float(l as f32 - r)))
                        }
                        _ => Err(self.runtime_error(&b.operator, "Invalid subtraction")),
                    },
                    TokenType::Slash => match (left.0, right.0) {
                        (Literal::Float(l), Literal::Float(r)) => Ok(Value(Literal::Float(l / r))),
                        (Literal::Int(l), Literal::Int(r)) => Ok(Value(Literal::Int(l / r))),
                        (Literal::Float(l), Literal::Int(r)) => {
                            Ok(Value(Literal::Float(l / r as f32)))
                        }
                        (Literal::Int(l), Literal::Float(r)) => {
                            Ok(Value(Literal::Float(l as f32 / r)))
                        }
                        _ => Err(self.runtime_error(&b.operator, "Invalid division")),
                    },
                    TokenType::Star => match (left.0, right.0) {
                        (Literal::Float(l), Literal::Float(r)) => Ok(Value(Literal::Float(l * r))),
                        (Literal::Int(l), Literal::Int(r)) => Ok(Value(Literal::Int(l * r))),
                        (Literal::Float(l), Literal::Int(r)) => {
                            Ok(Value(Literal::Float(l * r as f32)))
                        }
                        (Literal::Int(l), Literal::Float(r)) => {
                            Ok(Value(Literal::Float(l as f32 * r)))
                        }
                        _ => Err(self.runtime_error(&b.operator, "Invalid multiplication")),
                    },
                    TokenType::Plus => match (left.0, right.0) {
                        (Literal::Float(l), Literal::Float(r)) => Ok(Value(Literal::Float(l + r))),
                        (Literal::Int(l), Literal::Int(r)) => Ok(Value(Literal::Int(l + r))),
                        (Literal::Float(l), Literal::Int(r)) => {
                            Ok(Value(Literal::Float(l + r as f32)))
                        }
                        (Literal::Int(l), Literal::Float(r)) => {
                            Ok(Value(Literal::Float(l as f32 + r)))
                        }
                        // string concatenation!
                        (Literal::String(l), Literal::String(r)) => {
                            Ok(Value(Literal::String(l + &r)))
                        }
                        _ => Err(self.runtime_error(&b.operator, "Invalid addition")),
                    },
                    TokenType::Greater => match (left.0, right.0) {
                        (Literal::Float(l), Literal::Float(r)) => {
                            Ok(Value(Literal::Boolean(l > r)))
                        }
                        (Literal::Int(l), Literal::Int(r)) => Ok(Value(Literal::Boolean(l > r))),
                        (Literal::Float(l), Literal::Int(r)) => {
                            Ok(Value(Literal::Boolean(l > r as f32)))
                        }
                        (Literal::Int(l), Literal::Float(r)) => {
                            Ok(Value(Literal::Boolean(l as f32 > r)))
                        }
                        _ => Err(self.runtime_error(&b.operator, "Invalid comparison")),
                    },
                    TokenType::GreaterEqual => match (left.0, right.0) {
                        (Literal::Float(l), Literal::Float(r)) => {
                            Ok(Value(Literal::Boolean(l >= r)))
                        }
                        (Literal::Int(l), Literal::Int(r)) => Ok(Value(Literal::Boolean(l >= r))),
                        (Literal::Float(l), Literal::Int(r)) => {
                            Ok(Value(Literal::Boolean(l >= r as f32)))
                        }
                        (Literal::Int(l), Literal::Float(r)) => {
                            Ok(Value(Literal::Boolean(l as f32 >= r)))
                        }
                        _ => Err(self.runtime_error(&b.operator, "Invalid comparison")),
                    },
                    TokenType::Less => match (left.0, right.0) {
                        (Literal::Float(l), Literal::Float(r)) => {
                            Ok(Value(Literal::Boolean(l < r)))
                        }
                        (Literal::Int(l), Literal::Int(r)) => Ok(Value(Literal::Boolean(l < r))),
                        (Literal::Float(l), Literal::Int(r)) => {
                            Ok(Value(Literal::Boolean(l < r as f32)))
                        }
                        (Literal::Int(l), Literal::Float(r)) => {
                            Ok(Value(Literal::Boolean((l as f32) < r))) // needs parentheses, otherwise '<' is evaluated as a generic of f32
                        }
                        _ => Err(self.runtime_error(&b.operator, "Invalid comparison")),
                    },
                    TokenType::LessEqual => match (left.0, right.0) {
                        (Literal::Float(l), Literal::Float(r)) => {
                            Ok(Value(Literal::Boolean(l <= r)))
                        }
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
                    _ => Err(self.runtime_error(&b.operator, "Invalid binary operation")),
                }
            }
        }
    }

    fn runtime_error(&self, op: &Token, msg: &str) -> RuntimeError {
        RuntimeError {
            token: op.clone(),
            message: msg.to_owned(),
        }
    }
}

// The book implements a visitor pattern and instructs on writing a codegen tool to declare
// each node on the AST. Rust's enum types allow us to bypass this step completely.
pub fn print(expr: &Expr) -> String {
    match expr {
        Expr::Binary(b) => parenthesize(&b.operator.lexeme, &[&b.left, &b.right]),
        Expr::Unary(u) => parenthesize(&u.operator.lexeme, &[&u.right]),
        Expr::Grouping(g) => parenthesize("group", &[&g.expression]),
        Expr::Literal(l) => l.to_string(),
        Expr::Variable(t) => t.lexeme.to_owned(),
        Expr::Assignment(a) => String::from(&a.name.lexeme) + " = " + &print(&a.value),
        Expr::Logical(l) => parenthesize(&l.operator.lexeme, &[&l.left, &l.right]),
    }
}

/// Surround one or more expressions in parentheses.
fn parenthesize(lexeme: &str, exprs: &[&Expr]) -> String {
    let mut statement = String::from("(");
    statement.push_str(lexeme);
    for expr in exprs {
        statement.push_str(" ");
        statement.push_str(&print(expr));
    }
    statement.push_str(")");
    statement
}
