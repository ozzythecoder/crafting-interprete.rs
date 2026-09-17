use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::{expression::Expr, interpreter::Interpreter, statement::Stmt, token::Token};

pub struct Resolver {
    interpreter: Interpreter,
    scopes: Rc<RefCell<Vec<HashMap<String, bool>>>>,
    errors: Vec<ResolverError>,
}

#[derive(Debug)]
struct ResolverError {
    token: Token,
    msg: String,
}

impl Resolver {
    fn new(interpreter: Interpreter) -> Resolver {
        Resolver {
            interpreter,
            scopes: Rc::new(RefCell::new(vec![])),
            errors: vec![],
        }
    }

    fn resolve_block(&mut self, block: Vec<Stmt>) {
        self.begin_scope();
        for stmt in block {
            self.resolve_stmt(stmt);
        }
        self.end_scope();
    }

    fn resolve_stmt(&mut self, stmt: Stmt) {
        match stmt {
            Stmt::Var { name, initializer } => {
                self.declare(&name);
                if let Some(init) = initializer {
                    self.resolve_expr(&init);
                }
                self.define(&name);
            }
            Stmt::Function { name, params, body } => {
                self.declare(&name);
                self.define(&name);

                self.resolve_function(params, body);
            }
            Stmt::Expression(e) | Stmt::Print(e) => {
                self.resolve_expr(&e);
            }
            Stmt::Return { keyword: _, value } => {
                if let Some(val) = value {
                    self.resolve_expr(&val);
                }
            }
            Stmt::While { condition, body } => {
                self.resolve_expr(&condition);
                self.resolve_stmt(*body);
            }
            Stmt::If {
                condition,
                then_branch,
                else_branch,
            } => {
                self.resolve_expr(&condition);
                self.resolve_stmt(*then_branch);
                if let Some(el) = else_branch {
                    self.resolve_stmt(*el);
                }
            }
            Stmt::Block(b) => {
                self.resolve_block(b);
            }
        }
    }

    fn declare(&self, name: &Token) {
        if let Some(scope) = self.scopes.borrow_mut().last_mut() {
            scope.insert(name.lexeme.to_string(), false);
        };
    }

    fn define(&self, name: &Token) {
        if let Some(scope) = self.scopes.borrow_mut().last_mut() {
            scope.insert(name.lexeme.to_string(), true);
        };
    }

    fn resolve_expr(&mut self, expr: &Expr) {
        match expr {
            Expr::Variable(v) => {
                let err = if let Some(scope) = self.scopes.borrow_mut().last_mut()
                    && scope.get(&v.lexeme) == Some(&false)
                {
                    Some(
                        self.resolver_error(v, "Can't read local variable in its own initializer."),
                    )
                } else {
                    None
                };
                if let Some(e) = err {
                    self.errors.push(e);
                }

                self.resolve_local(expr.clone(), &v);
            }
            Expr::Assignment(a) => {
                self.resolve_expr(&a.value);
                self.resolve_local(Expr::Assignment(a.clone()), &a.name);
            }
            Expr::Binary(b) => {
                self.resolve_expr(&b.left);
                self.resolve_expr(&b.right);
            }
            Expr::Unary(u) => {
                self.resolve_expr(&u.right);
            }
            Expr::Logical(l) => {
                self.resolve_expr(&l.left);
                self.resolve_expr(&l.right);
            }
            Expr::Call(c) => {
                self.resolve_expr(&c.callee);
                for arg in c.args.iter() {
                    self.resolve_expr(&arg);
                }
            }
            Expr::Grouping(g) => {
                self.resolve_expr(&g.expression);
            }
            Expr::Literal(_) => (),
        }
    }

    fn resolve_local(&mut self, expr: Expr, name: &Token) {
        let scopes_iter = self.scopes.borrow().iter().rev(); // reversed to visit innermost scope first
        for (idx, scope) in scopes_iter.enumerate() {
            if scope.contains_key(&name.lexeme) {
                self.interpreter.resolve(expr, idx - 1);
                return;
            }
        }
    }

    fn resolve_function(&mut self, params: Vec<Token>, body: Vec<Stmt>) {
        self.begin_scope();
        for token in params {
            self.declare(&token);
            self.define(&token);
        }
        for stmt in body {
            self.resolve_stmt(stmt);
        }
        self.end_scope();
    }

    fn begin_scope(&self) {}

    fn end_scope(&self) {}

    fn resolver_error(&self, token: &Token, msg: &str) -> ResolverError {
        ResolverError {
            token: token.clone(),
            msg: msg.to_string(),
        }
    }

    fn push_resolver_error(&mut self, token: &Token, msg: &str) {
        self.errors.push(self.resolver_error(token, msg));
    }
}
