use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::{
    environment::Environment,
    expression::{
        Assignment, Callable, Class, ClassInstance, Expr, Function, IsTruthy, Literal,
        NativeFunction, TCallable, Value, Variable, to_callable_value,
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
        matches!(self, Self::Error(_))
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
    pub globals: Rc<RefCell<Environment>>,
    pub locals: Rc<RefCell<HashMap<usize, usize>>>,
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
            environment: Rc::new(RefCell::new(Environment::new(Some(global_cell.clone())))),
            globals: global_cell,
            locals: Rc::new(RefCell::new(HashMap::new())),
        }
    }

    pub fn interpret(&mut self, statements: &[Stmt]) -> Result<(), Interrupt<RuntimeError>> {
        for stmt in statements {
            match self.evaluate(stmt) {
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
            if let Some(s) = self.evaluate(stmt)
                && let Err(e) = s
            {
                match e {
                    Interrupt::Error(err) => {
                        return_value = Some(Err(err.wrap()));
                        break;
                    }
                    Interrupt::Return { value } => {
                        return_value = Some(Ok(value));
                        break;
                    }
                }
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
            Stmt::Class { name, methods: _ } => {
                self.environment.borrow_mut().define(
                    name,
                    to_callable_value(Callable::Class(Class {
                        name: name.lexeme.clone(),
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

    fn look_up_variable(&mut self, var: &Variable) -> Result<Value, Interrupt<RuntimeError>> {
        if let Some(distance) = self.locals.borrow().get(&var.id) {
            let this_env = self.environment.clone();
            let env = self.ancestor(*distance, this_env);
            env.borrow().get(&var.name)
        } else {
            self.globals.borrow().get(&var.name)
        }
    }

    fn assign_at(
        &self,
        distance: usize,
        var: &Assignment,
        val: Value,
    ) -> Result<(), Interrupt<RuntimeError>> {
        self.ancestor(distance, self.environment.clone())
            .borrow_mut()
            .assign(&var.name, &val)?;
        self.locals.borrow_mut().insert(var.id, distance);
        Ok(())
    }

    /// Get environment a certain number of generations up from current
    fn ancestor(&self, distance: usize, env: Rc<RefCell<Environment>>) -> Rc<RefCell<Environment>> {
        if distance == 0 {
            env
        } else {
            let enclosing = env
                .borrow()
                .enclosing
                .as_ref()
                .expect("Enclosing environment must exist")
                .clone();
            self.ancestor(distance, enclosing)
        }
    }

    pub fn resolve(&mut self, expr_id: usize, depth: usize) {
        self.locals.borrow_mut().insert(expr_id, depth);
    }

    pub fn call(
        &mut self,
        callee: Rc<RefCell<Callable>>,
        args: Vec<Value>,
        token: &Token,
    ) -> Result<Value, Interrupt<RuntimeError>> {
        match &*callee.borrow() {
            Callable::Function(f) => {
                // arity check
                self.check_arity(f.arity(), args.len(), token)?;

                // build a new environment whose parent is the function's closure
                let current_env = self.environment.clone();
                let mut new_env = Environment::new(Some(self.environment.clone()));

                // bind each param to each arg
                for (param, arg) in f.params.iter().zip(args) {
                    new_env.define(param, arg);
                }

                let env_cell = Rc::new(RefCell::new(new_env));

                // evaluate block
                let result = self.evaluate_block(&f.body, Some(env_cell.clone()));

                // revert environment
                let _ = std::mem::replace(&mut self.environment, current_env);

                // catch a Return value
                match result {
                    Some(Ok(val)) => Ok(val),
                    Some(Err(Interrupt::Error(e))) => Err(e.wrap()),
                    Some(Err(Interrupt::Return { value })) => Ok(value),
                    None => Ok(Value::Literal(Literal::Nil)),
                }
            }
            Callable::Native(f) => {
                if args.len() != f.arity {
                    return Err(self
                        .runtime_error(token, "Incorrect arity to native function")
                        .wrap());
                }
                (f.func)(self, args)
            }
            Callable::Class(c) => {
                self.check_arity(c.arity(), args.len(), token)?;

                // todo: args
                let instance = ClassInstance::new(Rc::new(c.clone()));

                Ok(Value::ClassInstance(instance))
            }
        }
    }

    pub fn check_arity(
        &self,
        arity: usize,
        len: usize,
        token: &Token,
    ) -> Result<(), Interrupt<RuntimeError>> {
        if arity != len {
            let msg = if len > arity {
                "Too many arguments to function."
            } else {
                "Too few arguments to function."
            };
            Err(self.runtime_error(token, msg).wrap())
        } else {
            Ok(())
        }
    }

    pub fn evaluate_expression(&mut self, expr: &Expr) -> Result<Value, Interrupt<RuntimeError>> {
        match expr {
            Expr::Literal(l) => Ok(Value::Literal(l.clone())),
            Expr::Grouping(g) => self.evaluate_expression(&g.expression),
            Expr::Variable(v) => self.look_up_variable(v),
            Expr::Assignment(a) => {
                let val = self.evaluate_expression(&a.value)?;

                if let Some(distance) = self.locals.borrow().get(&a.id) {
                    self.assign_at(*distance, a, val.clone())?;
                } else {
                    self.assign_at(0, a, val.clone())?;
                }
                Ok(val)
            }
            Expr::Call(c) => {
                let Value::Callable(v) = self.evaluate_expression(&c.callee)? else {
                    panic!("Non-callable passed to Call struct")
                };
                let mut args: Vec<Value> = vec![];
                for arg in c.args.iter() {
                    args.push(self.evaluate_expression(arg)?);
                }
                self.call(v, args, &c.paren)
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

                self.evaluate_expression(&l.right)
            }
            Expr::Unary(u) => {
                let right = self.evaluate_expression(&u.right)?;

                match u.operator.token_type {
                    // e.g. (-1)
                    TokenType::Minus => match right {
                        Value::Literal(Literal::Int(int)) => Ok(Value::Literal(Literal::Int(-int))),
                        Value::Literal(Literal::Float(float)) => {
                            Ok(Value::Literal(Literal::Float(-float)))
                        }
                        _ => Err(self
                            .runtime_error(&u.operator, "Cannot negate a non-number")
                            .wrap()),
                    },
                    // e.g. (!false), (-val)
                    TokenType::Bang => match right {
                        Value::Literal(Literal::False) => Ok(Value::Literal(Literal::True)),
                        Value::Literal(Literal::Nil) => Ok(Value::Literal(Literal::True)),
                        _ => Ok(Value::Literal(Literal::False)),
                    },
                    _ => Err(self
                        .runtime_error(&u.operator, "Invalid unary operator")
                        .wrap()),
                }
            }
            Expr::Binary(b) => {
                let left = self.evaluate_expression(&b.left)?;
                let right = self.evaluate_expression(&b.right)?;

                match b.operator.token_type {
                    TokenType::Minus => match (left, right) {
                        (Value::Literal(Literal::Float(l)), Value::Literal(Literal::Float(r))) => {
                            Ok(Value::Literal(Literal::Float(l - r)))
                        }
                        (Value::Literal(Literal::Int(l)), Value::Literal(Literal::Int(r))) => {
                            Ok(Value::Literal(Literal::Int(l - r)))
                        }
                        (Value::Literal(Literal::Float(l)), Value::Literal(Literal::Int(r))) => {
                            Ok(Value::Literal(Literal::Float(l - r as f32)))
                        }
                        (Value::Literal(Literal::Int(l)), Value::Literal(Literal::Float(r))) => {
                            Ok(Value::Literal(Literal::Float(l as f32 - r)))
                        }
                        _ => Err(self
                            .runtime_error(&b.operator, "Invalid subtraction")
                            .wrap()),
                    },
                    TokenType::Slash => match (left, right) {
                        (Value::Literal(Literal::Float(l)), Value::Literal(Literal::Float(r))) => {
                            if r == 0.0 {
                                Err(self
                                    .runtime_error(&b.operator, "Attempted to divide by zero")
                                    .wrap())
                            } else {
                                Ok(Value::Literal(Literal::Float(l / r)))
                            }
                        }
                        (Value::Literal(Literal::Int(l)), Value::Literal(Literal::Int(r))) => {
                            if r == 0 {
                                Err(self
                                    .runtime_error(&b.operator, "Attempted to divide by zero")
                                    .wrap())
                            } else {
                                Ok(Value::Literal(Literal::Int(l / r)))
                            }
                        }
                        (Value::Literal(Literal::Float(l)), Value::Literal(Literal::Int(r))) => {
                            if r == 0 {
                                Err(self
                                    .runtime_error(&b.operator, "Attempted to divide by zero")
                                    .wrap())
                            } else {
                                Ok(Value::Literal(Literal::Float(l / r as f32)))
                            }
                        }
                        (Value::Literal(Literal::Int(l)), Value::Literal(Literal::Float(r))) => {
                            if r == 0.0 {
                                Err(self
                                    .runtime_error(&b.operator, "Attempted to divide by zero")
                                    .wrap())
                            } else {
                                Ok(Value::Literal(Literal::Float(l as f32 / r)))
                            }
                        }
                        _ => Err(self.runtime_error(&b.operator, "Invalid division").wrap()),
                    },
                    TokenType::Star => match (left, right) {
                        (Value::Literal(Literal::Float(l)), Value::Literal(Literal::Float(r))) => {
                            Ok(Value::Literal(Literal::Float(l * r)))
                        }
                        (Value::Literal(Literal::Int(l)), Value::Literal(Literal::Int(r))) => {
                            Ok(Value::Literal(Literal::Int(l * r)))
                        }
                        (Value::Literal(Literal::Float(l)), Value::Literal(Literal::Int(r))) => {
                            Ok(Value::Literal(Literal::Float(l * r as f32)))
                        }
                        (Value::Literal(Literal::Int(l)), Value::Literal(Literal::Float(r))) => {
                            Ok(Value::Literal(Literal::Float(l as f32 * r)))
                        }
                        _ => Err(self
                            .runtime_error(&b.operator, "Invalid multiplication")
                            .wrap()),
                    },
                    TokenType::Plus => match (left, right) {
                        (Value::Literal(Literal::Float(l)), Value::Literal(Literal::Float(r))) => {
                            Ok(Value::Literal(Literal::Float(l + r)))
                        }
                        (Value::Literal(Literal::Int(l)), Value::Literal(Literal::Int(r))) => {
                            Ok(Value::Literal(Literal::Int(l + r)))
                        }
                        (Value::Literal(Literal::Float(l)), Value::Literal(Literal::Int(r))) => {
                            Ok(Value::Literal(Literal::Float(l + r as f32)))
                        }
                        (Value::Literal(Literal::Int(l)), Value::Literal(Literal::Float(r))) => {
                            Ok(Value::Literal(Literal::Float(l as f32 + r)))
                        }
                        // string concatenation!
                        (
                            Value::Literal(Literal::String(l)),
                            Value::Literal(Literal::String(r)),
                        ) => Ok(Value::Literal(Literal::String(l + &r))),
                        _ => Err(self.runtime_error(&b.operator, "Invalid addition").wrap()),
                    },
                    TokenType::Greater => match (left, right) {
                        (Value::Literal(Literal::Float(l)), Value::Literal(Literal::Float(r))) => {
                            Ok(Value::Literal(Literal::Boolean(l > r)))
                        }
                        (Value::Literal(Literal::Int(l)), Value::Literal(Literal::Int(r))) => {
                            Ok(Value::Literal(Literal::Boolean(l > r)))
                        }
                        (Value::Literal(Literal::Float(l)), Value::Literal(Literal::Int(r))) => {
                            Ok(Value::Literal(Literal::Boolean(l > r as f32)))
                        }
                        (Value::Literal(Literal::Int(l)), Value::Literal(Literal::Float(r))) => {
                            Ok(Value::Literal(Literal::Boolean(l as f32 > r)))
                        }
                        _ => Err(self.runtime_error(&b.operator, "Invalid comparison").wrap()),
                    },
                    TokenType::GreaterEqual => match (left, right) {
                        (Value::Literal(Literal::Float(l)), Value::Literal(Literal::Float(r))) => {
                            Ok(Value::Literal(Literal::Boolean(l >= r)))
                        }
                        (Value::Literal(Literal::Int(l)), Value::Literal(Literal::Int(r))) => {
                            Ok(Value::Literal(Literal::Boolean(l >= r)))
                        }
                        (Value::Literal(Literal::Float(l)), Value::Literal(Literal::Int(r))) => {
                            Ok(Value::Literal(Literal::Boolean(l >= r as f32)))
                        }
                        (Value::Literal(Literal::Int(l)), Value::Literal(Literal::Float(r))) => {
                            Ok(Value::Literal(Literal::Boolean(l as f32 >= r)))
                        }
                        _ => Err(self.runtime_error(&b.operator, "Invalid comparison").wrap()),
                    },
                    TokenType::Less => match (left, right) {
                        (Value::Literal(Literal::Float(l)), Value::Literal(Literal::Float(r))) => {
                            Ok(Value::Literal(Literal::Boolean(l < r)))
                        }
                        (Value::Literal(Literal::Int(l)), Value::Literal(Literal::Int(r))) => {
                            Ok(Value::Literal(Literal::Boolean(l < r)))
                        }
                        (Value::Literal(Literal::Float(l)), Value::Literal(Literal::Int(r))) => {
                            Ok(Value::Literal(Literal::Boolean(l < r as f32)))
                        }
                        (Value::Literal(Literal::Int(l)), Value::Literal(Literal::Float(r))) => {
                            Ok(Value::Literal(Literal::Boolean((l as f32) < r))) // needs parentheses, otherwise '<' is evaluated as a generic of f32
                        }
                        _ => Err(self.runtime_error(&b.operator, "Invalid comparison").wrap()),
                    },
                    TokenType::LessEqual => match (left, right) {
                        (Value::Literal(Literal::Float(l)), Value::Literal(Literal::Float(r))) => {
                            Ok(Value::Literal(Literal::Boolean(l <= r)))
                        }
                        (Value::Literal(Literal::Int(l)), Value::Literal(Literal::Int(r))) => {
                            Ok(Value::Literal(Literal::Boolean(l <= r)))
                        }
                        (Value::Literal(Literal::Float(l)), Value::Literal(Literal::Int(r))) => {
                            Ok(Value::Literal(Literal::Boolean(l <= r as f32)))
                        }
                        (Value::Literal(Literal::Int(l)), Value::Literal(Literal::Float(r))) => {
                            Ok(Value::Literal(Literal::Boolean(l as f32 <= r)))
                        }
                        _ => Err(self.runtime_error(&b.operator, "Invalid comparison").wrap()),
                    },
                    TokenType::BangEqual => Ok(Value::Literal(Literal::Boolean(left != right))),
                    TokenType::EqualEqual => Ok(Value::Literal(Literal::Boolean(left == right))),
                    _ => Err(self
                        .runtime_error(&b.operator, "Invalid binary operation")
                        .wrap()),
                }
            }
            Expr::Get { expr, name } => {
                if let Value::ClassInstance(c) = self.evaluate_expression(expr)? {
                    if let Some(val) = c.class.get(name) {
                        Ok(val)
                    } else {
                        let msg = format!("No property {} on class {}.", name.lexeme, c.class.name);
                        Err(self.runtime_error(name, &msg).wrap())
                    }
                } else {
                    Err(self
                        .runtime_error(name, "Only instances have properties.")
                        .wrap())
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
        Expr::Call(c) => todo!(),
        Expr::Binary(b) => parenthesize(&b.operator.lexeme, &[&b.left, &b.right]),
        Expr::Unary(u) => parenthesize(&u.operator.lexeme, &[&u.right]),
        Expr::Grouping(g) => parenthesize("group", &[&g.expression]),
        Expr::Literal(l) => l.to_string(),
        Expr::Variable(t) => t.name.lexeme.to_owned(),
        Expr::Assignment(a) => String::from(&a.name.lexeme) + " = " + &print(&a.value),
        Expr::Logical(l) => parenthesize(&l.operator.lexeme, &[&l.left, &l.right]),
    }
}

/// Surround one or more expressions in parentheses.
fn parenthesize(lexeme: &str, exprs: &[&Expr]) -> String {
    let mut statement = String::from("(");
    statement.push_str(lexeme);
    for expr in exprs {
        statement.push(' ');
        statement.push_str(&print(expr));
    }
    statement.push(')');
    statement
}
