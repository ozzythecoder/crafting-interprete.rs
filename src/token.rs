use std::fmt::Display;

use crate::expression::Literal;

#[derive(Debug, Clone, PartialEq, Copy)]
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

#[derive(Debug, Clone, PartialEq)]
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
