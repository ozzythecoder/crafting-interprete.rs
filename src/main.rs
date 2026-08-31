use std::{
    env::{self, Args},
    fs,
    io::{self, Read},
    path::PathBuf,
    process,
};

mod token;

struct Lox {
    had_error: bool
}

impl Lox {
    pub fn new() -> Self {
        Lox {had_error: false}
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
    let mut args = env::args();
    let mut lox = Lox::new();
    lox.main(&mut args);
}
