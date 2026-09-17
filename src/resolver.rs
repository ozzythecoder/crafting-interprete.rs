use std::collections::HashMap;

use crate::{expression::Expr, interpreter::Interpreter, statement::Stmt};

pub struct Resolver {
    interpreter: Interpreter,
    scopes: Vec<HashMap<String, bool>>,
}

impl Resolver {
    fn new(interpreter: Interpreter) -> Resolver {
        Resolver { interpreter, scopes: vec![] }
    }

    fn resolve_block(&self, block: Vec<Stmt>) {
        self.begin_scope();
        for stmt in block {
            self.resolve_stmt(stmt);
        }
        self.end_scope();
    }

    fn resolve_stmt(&self, stmt: Stmt) {}

    fn resolve_expr(&self, expr: Expr) {}

    fn begin_scope(&self) {}

    fn end_scope(&self) {}
}
