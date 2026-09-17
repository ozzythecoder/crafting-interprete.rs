use std::{cell::RefCell, rc::Rc};

use crate::{
    environment::Environment,
    expression::{
        Callable, Expr, Function, IsTruthy, Literal, NativeFunction, TCallable, Value,
        to_callable_value,
    },
    globals::clock_native,
    statement::Stmt,
    token::{Token, TokenType},
};

pub enum Interrupt<E> {
    Return { value: Value },
    Error(E),
}

impl<E> Interrupt<E> {
    fn is_err(&self) -> bool {
        match self {
            Self::Error(_) => true,
            _ => false,
        }
    }
}

#[derive(Debug, Clone)]
pub struct RuntimeError {
    pub token: Token,
    pub message: String,
}

impl RuntimeError {
    pub fn wrap(&self) -> Interrupt<RuntimeError> {
        Interrupt::Error(self.clone())
    }
}

pub struct Interpreter {
    pub environment: Rc<RefCell<Environment>>,
}

impl Interpreter {
    pub fn new() -> Self {
        let mut globals = Environment::new(None);

        globals.define_native(
            "clock",
            to_callable_value(Callable::Native(NativeFunction {
                name: "clock".into(),
                arity: 0,
                func: clock_native,
            })),
        );

        let global_cell = Rc::new(RefCell::new(globals));

        Interpreter {
            environment: Rc::new(RefCell::new(Environment::new(Some(global_cell)))),
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

    pub fn evaluate_block(
        &mut self,
        block: &[Stmt],
        environment: Option<Rc<RefCell<Environment>>>,
    ) -> Option<Result<Value, Interrupt<RuntimeError>>> {
        // replace previous environment with current
        let new_env = environment.unwrap_or_else(|| {
            Rc::new(RefCell::new(Environment::new(Some(Rc::clone(
                &self.environment,
            )))))
        });
        let prev_env = std::mem::replace(&mut self.environment, new_env);

        let mut return_value = None;

        // evaluate contents of block
        for stmt in block {
            match self.evaluate(stmt) {
                Some(s) => match s {
                    Err(e) => match e {
                        Interrupt::Error(err) => {
                            return Some(Err(err.wrap()));
                        }
                        Interrupt::Return { value } => {
                            return_value = Some(Ok(value));
                            break;
                        }
                    },
                    Ok(_) => (),
                },
                None => (),
            }
        }

        self.environment = prev_env;

        return_value
    }

    fn evaluate(&mut self, stmt: &Stmt) -> Option<Result<Value, Interrupt<RuntimeError>>> {
        match stmt {
            Stmt::Expression(e) => Some(self.evaluate_expression(e)),
            Stmt::Return { keyword: _, value } => {
                let val = if let Some(v) = value {
                    match self.evaluate_expression(v) {
                        Ok(o) => o,
                        Err(e) => {
                            return Some(Err(e));
                        }
                    }
                } else {
                    Value::Literal(Literal::Nil)
                };

                Some(Err(Interrupt::Return { value: val }))
            }
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
                        self.environment.borrow_mut().define(name, val);
                        None
                    }
                    Err(e) => Some(Err(e)),
                },
                None => {
                    self.environment
                        .borrow_mut()
                        .define(name, Value::Literal(Literal::Nil));
                    None
                }
            },
            Stmt::Function { name, params, body } => {
                self.environment.borrow_mut().define(
                    name,
                    to_callable_value(Callable::Function(Function {
                        params: params.clone(),
                        body: body.clone(),
                    })),
                );

                None
            }
            Stmt::Block(block) => {
                // evaluate in a new, enclosed environment
                let new_env = Environment::new(Some(Rc::clone(&self.environment)));
                self.evaluate_block(block, Some(Rc::new(RefCell::new(new_env))))
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
            Stmt::While { condition, body } => {
                loop {
                    match self.evaluate_expression(condition) {
                        Ok(val) => {
                            if !val.is_truthy() {
                                break;
                            }
                            self.evaluate(body);
                        }
                        Err(e) => return Some(Err(e)),
                    }
                }
                None
            }
        }
    }

    fn evaluate_expression(&mut self, expr: &Expr) -> Result<Literal, RuntimeError> {
        match expr {
            Expr::Literal(l) => Ok(l.clone()),
            Expr::Grouping(g) => self.evaluate_expression(&g.expression),
            Expr::Variable(v) => Ok(self.environment.get(v)?.clone()),
            Expr::Assignment(a) => {
                let val = self.evaluate_expression(&a.value)?;
                self.environment.assign(&a.name, &val)?;
                Ok(val)
            }
            Expr::Logical(l) => {
                let left = self.evaluate_expression(&l.left)?;

                if l.operator.token_type == TokenType::Or {
                    if left.is_truthy() {
                        return Ok(left);
                    }
                } else if !left.is_truthy() {
                    return Ok(left);
                }

                return self.evaluate_expression(&l.right);
            }
            Expr::Unary(u) => {
                let right = self.evaluate_expression(&u.right)?;

                match u.operator.token_type {
                    // e.g. (-1)
                    TokenType::Minus => match right {
                        Literal::Int(int) => Ok(Literal::Int(-1 * int)),
                        Literal::Float(float) => Ok(Literal::Float(-1.0 * float)),
                        _ => Err(RuntimeError {
                            token: u.operator.clone(),
                            message: "Cannot negate a non-number".to_owned(),
                        }),
                    },
                    // e.g. (!false), (-val)
                    TokenType::Bang => match right {
                        Literal::False => Ok(Literal::True),
                        Literal::Nil => Ok(Literal::True),
                        _ => Ok(Literal::False),
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
                    TokenType::Minus => match (left, right) {
                        (Literal::Float(l), Literal::Float(r)) => Ok(Literal::Float(l - r)),
                        (Literal::Int(l), Literal::Int(r)) => Ok(Literal::Int(l - r)),
                        (Literal::Float(l), Literal::Int(r)) => Ok(Literal::Float(l - r as f32)),
                        (Literal::Int(l), Literal::Float(r)) => Ok(Literal::Float(l as f32 - r)),
                        _ => Err(self.runtime_error(&b.operator, "Invalid subtraction")),
                    },
                    TokenType::Slash => match (left, right) {
                        (Literal::Float(l), Literal::Float(r)) => Ok(Literal::Float(l / r)),
                        (Literal::Int(l), Literal::Int(r)) => Ok(Literal::Int(l / r)),
                        (Literal::Float(l), Literal::Int(r)) => Ok(Literal::Float(l / r as f32)),
                        (Literal::Int(l), Literal::Float(r)) => Ok(Literal::Float(l as f32 / r)),
                        _ => Err(self.runtime_error(&b.operator, "Invalid division")),
                    },
                    TokenType::Star => match (left, right) {
                        (Literal::Float(l), Literal::Float(r)) => Ok(Literal::Float(l * r)),
                        (Literal::Int(l), Literal::Int(r)) => Ok(Literal::Int(l * r)),
                        (Literal::Float(l), Literal::Int(r)) => Ok(Literal::Float(l * r as f32)),
                        (Literal::Int(l), Literal::Float(r)) => Ok(Literal::Float(l as f32 * r)),
                        _ => Err(self.runtime_error(&b.operator, "Invalid multiplication")),
                    },
                    TokenType::Plus => match (left, right) {
                        (Literal::Float(l), Literal::Float(r)) => Ok(Literal::Float(l + r)),
                        (Literal::Int(l), Literal::Int(r)) => Ok(Literal::Int(l + r)),
                        (Literal::Float(l), Literal::Int(r)) => Ok(Literal::Float(l + r as f32)),
                        (Literal::Int(l), Literal::Float(r)) => Ok(Literal::Float(l as f32 + r)),
                        // string concatenation!
                        (Literal::String(l), Literal::String(r)) => Ok(Literal::String(l + &r)),
                        _ => Err(self.runtime_error(&b.operator, "Invalid addition")),
                    },
                    TokenType::Greater => match (left, right) {
                        (Literal::Float(l), Literal::Float(r)) => Ok(Literal::Boolean(l > r)),
                        (Literal::Int(l), Literal::Int(r)) => Ok(Literal::Boolean(l > r)),
                        (Literal::Float(l), Literal::Int(r)) => {
                            Ok(Literal::Boolean(l > r as f32))
                        }
                        (Literal::Int(l), Literal::Float(r)) => {
                            Ok(Literal::Boolean(l as f32 > r))
                        }
                        _ => Err(self.runtime_error(&b.operator, "Invalid comparison")),
                    },
                    TokenType::GreaterEqual => match (left, right) {
                        (Literal::Float(l), Literal::Float(r)) => Ok(Literal::Boolean(l >= r)),
                        (Literal::Int(l), Literal::Int(r)) => Ok(Literal::Boolean(l >= r)),
                        (Literal::Float(l), Literal::Int(r)) => {
                            Ok(Literal::Boolean(l >= r as f32))
                        }
                        (Literal::Int(l), Literal::Float(r)) => {
                            Ok(Literal::Boolean(l as f32 >= r))
                        }
                        _ => Err(self.runtime_error(&b.operator, "Invalid comparison")),
                    },
                    TokenType::Less => match (left, right) {
                        (Literal::Float(l), Literal::Float(r)) => Ok(Literal::Boolean(l < r)),
                        (Literal::Int(l), Literal::Int(r)) => Ok(Literal::Boolean(l < r)),
                        (Literal::Float(l), Literal::Int(r)) => {
                            Ok(Literal::Boolean(l < r as f32))
                        }
                        (Literal::Int(l), Literal::Float(r)) => {
                            Ok(Literal::Boolean((l as f32) < r)) // needs parentheses, otherwise '<' is evaluated as a generic of f32
                        }
                        _ => Err(self.runtime_error(&b.operator, "Invalid comparison")),
                    },
                    TokenType::LessEqual => match (left, right) {
                        (Literal::Float(l), Literal::Float(r)) => Ok(Literal::Boolean(l <= r)),
                        (Literal::Int(l), Literal::Int(r)) => Ok(Literal::Boolean(l <= r)),
                        (Literal::Float(l), Literal::Int(r)) => {
                            Ok(Literal::Boolean(l <= r as f32))
                        }
                        (Literal::Int(l), Literal::Float(r)) => {
                            Ok(Literal::Boolean(l as f32 <= r))
                        }
                        _ => Err(RuntimeError {
                            token: b.operator.clone(),
                            message: "Invalid comparison".to_owned(),
                        }),
                    },
                    TokenType::BangEqual => Ok(Literal::Boolean(left != right)),
                    TokenType::EqualEqual => Ok(Literal::Boolean(left == right)),
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
