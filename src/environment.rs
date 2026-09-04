use std::collections::HashMap;

use crate::{expression::Literal, interpreter::RuntimeError, token::Token};

pub struct Environment {
    values: HashMap<String, Literal>,
}

impl Environment {
    pub fn new() -> Self {
        Environment {
            values: HashMap::new(),
        }
    }

    pub fn define(&mut self, token: &Token, value: Literal) {
        self.values.insert(token.lexeme.to_owned(), value);
    }

    pub fn get(&mut self, token: &Token) -> Result<&Literal, RuntimeError> {
        match self.values.get(&token.lexeme) {
            Some(l) => Ok(l),
            None => Err(RuntimeError {
                token: token.clone(),
                message: String::from("Undeclared variable"),
            }),
        }
    }

    pub fn assign(&mut self, token: &Token, value: &Literal) -> Result<(), RuntimeError> {
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
