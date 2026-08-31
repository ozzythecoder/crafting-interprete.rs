use std::fmt::Display;

#[derive(Debug, Clone)]
pub enum TokenType {
    // single-character tokens
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    Comma,
    Dot,
    Minus,
    Plus,
    SemiColon,
    Slash,
    Star,

    // one- or two-character tokens
    Bang,
    BangEqual,
    Equal,
    EqualEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,

    // literals
    Identifier,
    String,
    Number,

    // keywords
    And,
    Class,
    Else,
    False,
    Fun,
    For,
    If,
    Nil,
    Or,
    Print,
    Return,
    Super,
    This,
    True,
    Var,
    While,

    EOF,
}

impl Display for TokenType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Clone, Debug)]
pub enum Literal {
    String(String),
    Int(i32),
    Float(f32),
}

impl Display for Literal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

#[derive(Debug, Clone)]
pub struct Token {
    pub token_type: TokenType,
    pub lexeme: String,
    pub line: Option<usize>,
    pub literal: Option<Literal>,
}

impl ToString for Token {
    fn to_string(&self) -> String {
        let line = if let Some(l) = self.line {
            l.to_string()
        } else {
            "".to_owned()
        };
        format!(
            "{} {} {:?} {}",
            self.token_type, self.lexeme, self.literal, line
        )
    }
}

struct Scanner {
    source: Vec<char>,
    tokens: Vec<Token>,
    start: usize,
    current: usize,
    line: usize,
}

impl Scanner {
    pub fn new(source: String) -> Self {
        Scanner {
            source: source.chars().collect(),
            start: 0,
            current: 0,
            line: 1,
            tokens: vec![],
        }
    }

    pub fn scan_tokens(&mut self) -> Vec<Token> {
        while !self.is_at_end() {
            self.start = self.current;
            self.scan_token();
        }
        self.tokens.push(Token {
            token_type: TokenType::EOF,
            lexeme: String::from(""),
            line: None,
            literal: None,
        });
        self.tokens.clone()
    }

    fn scan_token(&mut self) {
        let c = self.advance();
        match c {
            '(' => self.add_token(TokenType::LeftParen, None),
            ')' => self.add_token(TokenType::RightParen, None),
            '{' => self.add_token(TokenType::LeftBrace, None),
            '}' => self.add_token(TokenType::RightBrace, None),
            ',' => self.add_token(TokenType::Comma, None),
            '.' => self.add_token(TokenType::Dot, None),
            '-' => self.add_token(TokenType::Minus, None),
            '+' => self.add_token(TokenType::Plus, None),
            ';' => self.add_token(TokenType::SemiColon, None),
            '*' => self.add_token(TokenType::Star, None),
            '!' => {
                let token = if self.match_next('=') {
                    TokenType::BangEqual
                } else {
                    TokenType::Bang
                };
                self.add_token(token, None);
            }
            '=' => {
                let token = if self.match_next('=') {
                    TokenType::EqualEqual
                } else {
                    TokenType::Equal
                };
                self.add_token(token, None);
            }
            '<' => {
                let token = if self.match_next('=') {
                    TokenType::LessEqual
                } else {
                    TokenType::Less
                };
                self.add_token(token, None);
            }
            '>' => {
                let token = if self.match_next('=') {
                    TokenType::GreaterEqual
                } else {
                    TokenType::Greater
                };
                self.add_token(token, None);
            }
            '/' => {
                // two forward slashes is a comment
                // comment ends at newline
                if self.match_next('/') {
                    while self.peek() != '\n' && !self.is_at_end() {
                        self.advance();
                    }
                } else {
                    self.add_token(TokenType::Slash, None);
                }
            }
            '"' => self.consume_string(),
            // whitespace does nothing
            ' ' => (),
            '\r' => (),
            '\t' => (),
            // newline does not add a new token
            '\n' => self.line += 1,
            c => {
                if self.is_digit(c) {
                    self.consume_number();
                } else if self.is_alpha(c) {
                    self.consume_identifier();
                } else {
                    todo!() // propagate error back to Lox class
                }
            }
        };
    }

    fn keyword(&self, text: &str) -> Option<TokenType> {
        match text {
            "and" => Some(TokenType::And),
            "class" => Some(TokenType::Class),
            "else" => Some(TokenType::Else),
            "false" => Some(TokenType::False),
            "for" => Some(TokenType::For),
            "fun" => Some(TokenType::Fun),
            "if" => Some(TokenType::If),
            "nil" => Some(TokenType::Nil),
            "or" => Some(TokenType::Or),
            "print" => Some(TokenType::Print),
            "return" => Some(TokenType::Return),
            "super" => Some(TokenType::Super),
            "this" => Some(TokenType::This),
            "true" => Some(TokenType::True),
            "var" => Some(TokenType::Var),
            "while" => Some(TokenType::While),
            _ => None,
        }
    }

    fn advance(&mut self) -> char {
        self.current += 1;
        self.source[self.current]
    }

    fn add_token(&mut self, token_type: TokenType, literal: Option<Literal>) {
        self.tokens.push(Token {
            token_type,
            literal,
            lexeme: self.source[self.start..self.current].iter().collect(),
            line: Some(self.line),
        });
    }

    fn match_next(&mut self, expected: char) -> bool {
        if self.is_at_end() || self.source[self.current] != expected {
            false
        } else {
            self.current += 1;
            true
        }
    }

    fn peek(&self) -> char {
        if self.is_at_end() {
            '\0'
        } else {
            self.source[self.current]
        }
    }

    fn peek_next(&self) -> char {
        if self.current + 1 >= self.source.len() {
            '\0'
        } else {
            self.source[self.current + 1]
        }
    }

    fn consume_string(&mut self) {
        while self.peek() != '"' && !self.is_at_end() {
            if self.peek() == '\n' {
                self.line += 1;
            };
            self.advance();
        }

        if self.is_at_end() {
            todo!(); // error for unterminated string
        }

        self.advance();
        let chars = &self.source[self.start + 1..self.current - 1];
        let s = chars.iter().map(|e| e.to_owned()).collect::<String>();
        self.add_token(TokenType::String, Some(Literal::String(s)));
    }

    fn consume_number(&mut self) {
        while self.is_digit(self.peek()) {
            self.advance();
        }

        if self.peek() == '.' && self.is_digit(self.peek_next()) {
            self.advance();

            while self.is_digit(self.peek()) {
                self.advance();
            }
        }

        let num = String::from(
            self.source[self.start..self.current]
                .iter()
                .collect::<String>(),
        );

        match num.parse::<f32>() {
            Ok(v) => {
                self.add_token(TokenType::Number, Some(Literal::Float(v)));
            }
            Err(e) => {
                todo!();
            }
        };
    }

    fn consume_identifier(&mut self) {
        while self.is_alphanumeric(self.peek()) {
            self.advance();
        }

        let text = &self.source[self.start..self.current]
            .iter()
            .collect::<String>();
        let token_type = self.keyword(text).unwrap_or(TokenType::Identifier);
        self.add_token(token_type, None);
    }

    fn is_alpha(&self, c: char) -> bool {
        (c >= 'a' && c <= 'z') || (c >= 'A' && c <= 'Z') || c == '_'
    }

    fn is_digit(&self, c: char) -> bool {
        c >= '0' && c <= '9'
    }

    fn is_alphanumeric(&self, c: char) -> bool {
        self.is_alpha(c) || self.is_digit(c)
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.source.len()
    }
}
