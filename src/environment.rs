use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::{expression::Value, interpreter::{Interrupt, RuntimeError}, token::Token};

#[derive(Default)]
pub struct Environment {
    pub values: HashMap<String, Value>,
    pub enclosing: Option<Rc<RefCell<Environment>>>,
}

impl Environment {
    pub fn new(enclosing: Option<Rc<RefCell<Environment>>>) -> Self {
        Environment {
            values: HashMap::new(),
            enclosing,
        }
    }

    pub fn define(&mut self, token: &Token, value: Value) {
        self.values.insert(token.lexeme.to_owned(), value);
    }

    pub fn define_native(&mut self, name: &str, value: Value) {
        self.values.insert(name.to_owned(), value);
    }

    pub fn get(&self, token: &Token) -> Result<Value, Interrupt<RuntimeError>> {
        if let Some(v) = self.get_var(token) {
            Ok(v.clone())
        } else if let Some(env) = &self.enclosing {
            env.borrow().get(token)
        } else {
            Err(self.undefined_var(token).wrap())
        }
    }

    fn get_var(&self, token: &Token) -> Option<&Value> {
        self.values.get(&token.lexeme)
    }

    pub fn assign(&mut self, token: &Token, value: &Value) -> Result<(), Interrupt<RuntimeError>> {
        if self.get_var(token).is_some() {
            self.assign_var(token, value)
        } else if self.enclosing.is_some() {
            let env = self.enclosing.as_mut().expect("Quantum nonsense");
            env.borrow_mut().assign(token, value)
        } else {
            Err(self.undefined_var(token).wrap())
        }
    }

    fn assign_var(&mut self, token: &Token, value: &Value) -> Result<(), Interrupt<RuntimeError>> {
        if self.values.contains_key(&token.lexeme) {
            self.values.insert(token.lexeme.to_owned(), value.clone());
            Ok(())
        } else {
            Err(self.undefined_var(token).wrap())
        }
    }

    fn undefined_var(&self, token: &Token) -> RuntimeError {
        RuntimeError {
            token: token.clone(),
            message: String::from("Undefined variable"),
        }
    }
}
