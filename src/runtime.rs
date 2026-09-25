use std::{
    env::Args,
    fs,
    io::{BufRead, Write},
    path::PathBuf,
    process,
};

use crate::{
    interpreter::{Interpreter, RuntimeError},
    parser::Parser,
    resolver::Resolver,
    scanner::Scanner,
};

pub struct Lox {
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
            Ok(bytes) => {
                self.run(bytes);
                if self.had_error {
                    process::exit(65);
                }
                if self.had_runtime_error {
                    process::exit(70);
                }
            }
            Err(e) => {
                println!("Error occurred when reading file at {}", &path);
                println!("{:?}", e);
                process::exit(65);
            }
        }
    }

    fn run_prompt(&mut self) {
        clear_screen();
        println!("Welcome to the Lox REPL!");
        println!("Press Ctrl+D to exit.\n");

        let stdin = std::io::stdin();
        loop {
            print!("> ");
            std::io::stdout().flush().unwrap();

            let mut line = String::new();
            match stdin.lock().read_line(&mut line) {
                Ok(0) => {
                    process::exit(0);
                }
                Ok(_) => {
                    self.run(line.into_bytes());
                    self.had_error = false;
                }
                Err(_) => self.error("Could not read stdin".to_string(), None),
            };
        }
    }

    fn error(&mut self, msg: String, line: Option<usize>) {
        if let Some(l) = line {
            println!("[{:?}] Error: {}", l, msg);
        } else {
            println!("Error: {}", msg);
        }
        self.had_error = true;
    }

    fn runtime_error(&mut self, error: RuntimeError) {
        if let Some(line) = error.token.line {
            println!("[{line}] Runtime Error: {}", error.message);
        } else {
            println!("Runtime Error: {}", error.message);
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
        let ast = Parser::new(tokens).parse();
        let mut resolver = Resolver::new(Interpreter::new());

        resolver.resolve(&ast);

        if !resolver.errors.is_empty() {
            for e in resolver.errors.iter() {
                println!("Resolver Error: {:?}", e);
            }
            return;
        }

        match resolver.into_interpreter().interpret(&ast) {
            Ok(_) => (),
            Err(e) => {
                if let crate::interpreter::Interrupt::Error(e) = e {
                    self.runtime_error(e)
                }
            }
        };
    }
}

fn clear_screen() {
    print!("\x1B[2J");
}
