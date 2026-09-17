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
                    self.resolve_expr(init);
                }
                self.define(&name);
            }
            _ => todo!(),
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

    fn resolve_expr(&mut self, expr: Expr) {
        match expr {
            Expr::Variable(v) => {
                if let Some(scope) = self.scopes.borrow_mut().last_mut()
                    && scope.get(&v.lexeme) == Some(&false)
                {
                    let err = self.resolver_error(v, "Can't read local variable in its own initializer.");
                    println!("{:?}", err);
                }
            }
            _ => todo!(),
        }
    }

    fn resolve_local(&mut self, expr: Expr, name: Token) {
        let scopes_iter = self.scopes.borrow().iter().rev(); // reversed to visit innermost scope first
        for (idx, scope) in scopes_iter.enumerate() {
            if scope.contains_key(&name.lexeme) {
                self.interpreter.resolve(expr, idx - 1);
                return;
            }
        }
    }

    fn begin_scope(&self) {}

    fn end_scope(&self) {}

    fn resolver_error(&self, token: Token, msg: &str) -> ResolverError {
        ResolverError {
            token,
            msg: msg.to_string(),
        }
    }

    fn push_resolver_error(&mut self, token: Token, msg: &str) {
        self.errors.push(self.resolver_error(token, msg));
    }
}
