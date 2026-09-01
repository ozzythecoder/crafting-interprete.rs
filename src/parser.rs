use crate::{
    expression::{Binary, Expr, Grouping, Literal, Unary},
    token::{Token, TokenType},
};

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

pub struct ParseError {
    token: Token,
    message: String,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser { tokens, current: 0 }
    }

    pub fn parse(&mut self) -> Option<Expr> {
        match self.expression() {
            Ok(exp) => Some(exp),
            Err(e) => {
                // self.synchronize
                None
            }
        }
    }

    fn expression(&mut self) -> Result<Expr, ParseError> {
        self.equality()
    }

    fn equality(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.comparison()?;

        while self.match_expr(&[TokenType::BangEqual, TokenType::EqualEqual]) {
            let operator = self.previous();
            let right = self.comparison()?;
            expr = Expr::Binary(Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            });
        }
        Ok(expr)
    }

    fn comparison(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.term()?;

        while self.match_expr(&[
            TokenType::Greater,
            TokenType::GreaterEqual,
            TokenType::Less,
            TokenType::LessEqual,
        ]) {
            let operator = self.previous();
            let right = self.term()?;
            expr = Expr::Binary(Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            })
        }
        Ok(expr)
    }

    fn term(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.factor()?;

        while self.match_expr(&[TokenType::Minus, TokenType::Plus]) {
            let operator = self.previous();
            let right = self.factor()?;
            expr = Expr::Binary(Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            })
        }

        Ok(expr)
    }

    fn factor(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.unary()?;

        while self.match_expr(&[TokenType::Slash, TokenType::Star]) {
            let operator = self.previous();
            let right = self.unary()?;
            expr = Expr::Binary(Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            });
        }

        Ok(expr)
    }

    fn unary(&mut self) -> Result<Expr, ParseError> {
        if self.match_expr(&[TokenType::Bang, TokenType::Minus]) {
            let operator = self.previous();
            let right = self.unary()?;
            Ok(Expr::Unary(Unary {
                operator,
                right: Box::new(right),
            }))
        } else {
            self.primary()
        }
    }

    /// Represents all literals and expression groups. The grammatical bottom.
    fn primary(&mut self) -> Result<Expr, ParseError> {
        if self.match_expr(&[TokenType::False]) {
            Ok(Expr::Literal(Literal::False))
        } else if self.match_expr(&[TokenType::True]) {
            Ok(Expr::Literal(Literal::True))
        } else if self.match_expr(&[TokenType::Nil]) {
            Ok(Expr::Literal(Literal::Nil))
        } else if self.match_expr(&[TokenType::String, TokenType::Number]) {
            Ok(Expr::Literal(self.previous().literal.unwrap()))
        } else if self.match_expr(&[TokenType::LeftParen]) {
            let expr = self.expression()?;
            self.consume(TokenType::RightParen, "Expect ')' after expression.")?;
            Ok(Expr::Grouping(Grouping {
                expression: Box::new(expr),
            }))
        } else {
            // Exhausted all options for valid syntax - report an error
            let message = String::from("Expression expected.");
            Err(ParseError {
                token: self.peek().clone(),
                message,
            })
        }
    }

    /// Checks the type of the next token. If it matches the passed `token_type`, it advances the parser
    /// and returns the next token. If they do not match, returns an error with the passed `message`.
    fn consume(&mut self, token_type: TokenType, message: &str) -> Result<Token, ParseError> {
        if self.check(token_type) {
            Ok(self.advance())
        } else {
            Err(ParseError {
                token: self.peek().clone(),
                message: message.to_owned(),
            })
        }
    }

    /// Returns `true` if the next token's type is one of the passed token types.
    /// For matching on a single token, use `self.check()`.
    fn match_expr(&mut self, tokens: &[TokenType]) -> bool {
        for token_type in tokens {
            if self.check(*token_type) {
                self.advance();
                return true;
            }
        }
        return false;
    }

    /// Returns `true` if the next token's type is equal to the passed token type, and is not EOF.
    fn check(&self, token_type: TokenType) -> bool {
        if self.is_at_end() {
            false
        } else {
            self.peek().token_type == token_type
        }
    }

    /// Consumes the current token, and steps the parser forward to the next token.
    fn advance(&mut self) -> Token {
        if !self.is_at_end() {
            self.current += 1;
        }
        self.previous()
    }

    fn is_at_end(&self) -> bool {
        self.peek().token_type == TokenType::EOF
    }

    /// Returns the next unconsumed token.
    fn peek(&self) -> &Token {
        &self.tokens[self.current]
    }

    /// Returns the last consumed token.
    fn previous(&self) -> Token {
        self.tokens[self.current - 1].clone()
    }
}
