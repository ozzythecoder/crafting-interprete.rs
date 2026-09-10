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
        if let Some(env) = &self.enclosing {
            env.get(token)
        } else {
            self.get_var(token)
        }
    }

    pub fn get_var(&self, token: &Token) -> Result<&Literal, RuntimeError> {
        match self.values.get(&token.lexeme) {
            Some(l) => Ok(l),
            None => Err(RuntimeError {
                token: token.clone(),
                message: String::from("Undeclared variable"),
            }),
        }
    }

    pub fn assign(&mut self, token: &Token, value: &Literal) -> Result<(), RuntimeError> {
        if let Some(env) = &mut self.enclosing {
            env.assign(token, value)
        } else {
            self.assign_var(token, value)
        }
    }

    fn assign_var(&mut self, token: &Token, value: &Literal) -> Result<(), RuntimeError> {
        if self.values.contains_key(&token.lexeme) {
            self.values.insert(token.lexeme.to_owned(), value.clone());
            Ok(())
        } else {
            Err(RuntimeError {
                token: token.clone(),
                message: String::from("Undefined variable"),
            })
        }
    }
}
