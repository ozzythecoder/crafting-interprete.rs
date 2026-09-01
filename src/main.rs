use std::{
    env::Args,
    fs,
    io::{self, Read},
    path::PathBuf,
    process,
};

use crate::{expression::Expr, parser::Parser, scanner::Scanner, token::Token};

mod expression;
mod parser;
mod scanner;
mod token;

struct Lox {
    had_error: bool,
    expr: Option<Expr>,
}

impl Lox {
    pub fn new(tokens: Vec<Token>) -> Self {
        let mut parser = Parser::new(tokens);
        Lox {
            had_error: false,
            expr: parser.parse(),
        }
    }

    pub fn main(&mut self, args: &mut Args) {
        if args.len() > 1 {
            println!("Usage: jlox [script");
            process::exit(64);
        } else {
            match args.next() {
                Some(path) => self.run_file(path),
                None => self.run_prompt(),
            };
        }
    }

    fn run_file(&mut self, path: String) {
        match fs::read(PathBuf::from(&path)) {
            Ok(bytes) => self.run(bytes),
            Err(e) => {
                println!("Error occurred when reading file at {}", &path);
                println!("{:?}", e);
                process::exit(65);
            }
        }
    }

    fn run_prompt(&mut self) {
        loop {
            let mut input = Vec::<u8>::new();
            match io::stdin().read(&mut input) {
                Ok(_) => self.run(input),
                Err(_) => self.error(format!("Error: could not read stdin"), None),
            }
        }
    }

    fn error(&mut self, msg: String, line: Option<u8>) {
        println!("{}", msg);
        if let Some(l) = line {
            println!("\tOccurred on line {}", l);
        }
        self.had_error = true;
    }

    fn run(&mut self, bytes: Vec<u8>) {}
}

fn main() {
    print_ast();
}

fn print_ast() {
    let source = "( 210 * 4 ) / 2";
    let mut scanner = Scanner::new(source.to_owned());
    let tokens = scanner.scan_tokens();
    let mut parser = Parser::new(tokens);

    if let Some(ast) = parser.parse() {
        println!("{:#?}", ast);
    } else {
        println!("Parser returned None.");
    }
}
