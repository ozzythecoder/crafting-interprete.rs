use std::collections::HashMap;

use crate::{expression::Literal, interpreter::RuntimeError, token::Token};

#[derive(Default)]
pub struct Environment {
    pub values: HashMap<String, Literal>,
    pub enclosing: Option<Box<Environment>>,
}

impl Environment {
    pub fn new(enclosing: Option<Environment>) -> Self {
        Environment {
            values: HashMap::new(),
            enclosing: if let Some(e) = enclosing {
                Some(Box::new(e))
            } else {
                None
            },
        }
    }

    pub fn define(&mut self, token: &Token, value: Literal) {
        self.values.insert(token.lexeme.to_owned(), value);
    }

    pub fn get(&self, token: &Token) -> Result<&Literal, RuntimeError> {
        if let Some(v) = self.get_var(token) {
            Ok(v)
        } else if let Some(env) = &self.enclosing {
            env.get(token)
        } else {
            Err(self.undefined_var(token))
        }
    }

    fn get_var(&self, token: &Token) -> Option<&Literal> {
        self.values.get(&token.lexeme)
    }

    pub fn assign(&mut self, token: &Token, value: &Literal) -> Result<(), RuntimeError> {
        if let Some(_) = self.get_var(token) {
            self.assign_var(token, value)
        } else if self.enclosing.is_some() {
            let env = self.enclosing.as_mut().expect("Quantum nonsense");
            env.assign(token, value)
        } else {
            Err(self.undefined_var(token))
        }
    }

    fn assign_var(&mut self, token: &Token, value: &Literal) -> Result<(), RuntimeError> {
        if self.values.contains_key(&token.lexeme) {
            self.values.insert(token.lexeme.to_owned(), value.clone());
            Ok(())
        } else {
            Err(self.undefined_var(token))
        }
    }

    fn undefined_var(&self, token: &Token) -> RuntimeError {
        RuntimeError {
            token: token.clone(),
            message: String::from("Undefined variable"),
        }
    }
}
