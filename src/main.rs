use std::{
    env::{self, Args},
    fs,
    io::{self, BufRead, Read, Write},
    path::PathBuf,
    process,
};

use crate::{
    expression::{Expr, RuntimeError, evaluate, interpret},
    parser::Parser,
    scanner::Scanner,
    token::Token,
};

mod expression;
mod parser;
mod scanner;
mod token;

struct Lox {
    had_error: bool,
    had_runtime_error: bool,
}

impl Lox {
    pub fn new() -> Self {
        Lox {
            had_error: false,
            had_runtime_error: false,
        }
    }

    pub fn main(&mut self, args: &mut Args) {
        dbg!(&args);
        if args.len() > 2 {
            println!("Usage: jlox [script]");
            process::exit(64);
        } else {
            args.next(); // discard first arg, which is the project filename
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
        let stdin = io::stdin();
        loop {
            print!("> ");
            io::stdout().flush().unwrap();

            let mut line = String::new();
            match stdin.lock().read_line(&mut line) {
                Ok(0) => {
                    process::exit(0);
                }
                Ok(_) => {
                    self.run(line.into_bytes());
                    self.had_error = false;
                }
                Err(_) => self.error(format!("Error: could not read stdin"), None),
            };
        }
    }

    fn error(&mut self, msg: String, line: Option<usize>) {
        println!("{}", msg);
        if let Some(l) = line {
            println!("\tOccurred on line {}", l);
        }
        self.had_error = true;
    }

    fn runtime_error(&mut self, error: RuntimeError) {
        if let Some(line) = error.token.line {
            println!("[{line}] Runtime Error:, {}", error.message);
        } else {
            println!("Runtime Error:, {}", error.message);
        }
        self.had_runtime_error = true;
    }

    fn run(&mut self, bytes: Vec<u8>) {
        let source = match String::from_utf8(bytes) {
            Ok(src) => src,
            Err(e) => {
                self.had_error = true;
                println!("Invalid UTF-8: {e}");
                return;
            }
        };
        let tokens = Scanner::new(source).scan_tokens();
        let ast = match Parser::new(tokens).parse() {
            Some(tree) => tree,
            None => {
                self.error("Error in parser".to_owned(), None);
                return;
            }
        };
        match interpret(ast) {
            Ok(result) => {
                println!("{:?}", result);
            }
            Err(e) => {
                self.runtime_error(e);
            }
        };
    }
}

fn main() {
    let mut args = env::args();
    Lox::new().main(&mut args);
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
