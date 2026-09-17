use crate::runtime::Lox;
use std::env;

mod environment;
mod expression;
mod globals;
mod interpreter;
mod parser;
mod resolver;
mod runtime;
mod scanner;
mod statement;
mod token;

fn main() {
    let mut args = env::args();
    Lox::new().main(&mut args);
}
