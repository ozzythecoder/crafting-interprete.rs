use crate::runtime::Lox;
use std::env;

mod expression;
mod parser;
mod runtime;
mod scanner;
mod token;

fn main() {
    let mut args = env::args();
    Lox::new().main(&mut args);
}
