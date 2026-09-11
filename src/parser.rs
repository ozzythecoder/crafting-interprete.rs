use crate::{
    expression::{Assignment, Binary, Call, Expr, Grouping, Literal, Logical, Unary},
    statement::Stmt,
    token::{Token, TokenType},
};

pub struct Parser {
    tokens: Vec<Token>,
    statements: Vec<Stmt>,
    current: usize,
}

#[derive(Debug)]
pub struct ParseError {
    pub token: Token,
    pub message: String,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser {
            tokens,
            statements: vec![],
            current: 0,
        }
    }

    pub fn parse(&mut self) -> Vec<Stmt> {
        while !self.is_at_end() {
            match self.declaration() {
                Ok(s) => self.statements.push(s),
                Err(e) => {
                    self.parse_error(e);
                    self.advance();
                }
            }
        }
        self.statements.clone()
    }

    fn declaration(&mut self) -> Result<Stmt, ParseError> {
        if self.check(TokenType::Var) {
            self.var_declaration()
        } else {
            self.statement()
        }
    }

    fn var_declaration(&mut self) -> Result<Stmt, ParseError> {
        self.advance(); // consume `var` keyword
        let name = self.consume(TokenType::Identifier, "Expected variable name.")?;
        let initializer = if self.match_expr(&[TokenType::Equal]) {
            Some(self.expression()?)
        } else {
            None
        };
        self.consume(
            TokenType::SemiColon,
            "Expected ';' after variable declaration.",
        )?;
        Ok(Stmt::Var { name, initializer })
    }

    fn statement(&mut self) -> Result<Stmt, ParseError> {
        if self.match_expr(&[TokenType::For]) {
            self.for_statement()
        } else if self.match_expr(&[TokenType::Print]) {
            self.print_statement()
        } else if self.match_expr(&[TokenType::If]) {
            self.if_statement()
        } else if self.match_expr(&[TokenType::While]) {
            self.while_statement()
        } else if self.match_expr(&[TokenType::LeftBrace]) {
            self.block_statement()
        } else {
            self.expression_statement()
        }
    }

    /// Desugars a for loop into a while loop.
    fn for_statement(&mut self) -> Result<Stmt, ParseError> {
        self.consume(TokenType::LeftParen, "Expected '(' after 'for'.")?;

        let initializer = if self.match_expr(&[TokenType::SemiColon]) {
            None
        } else if self.check(TokenType::Var) {
            Some(self.var_declaration()?)
        } else {
            Some(self.expression_statement()?)
        };

        let mut condition = if !self.match_expr(&[TokenType::RightParen]) {
            Some(self.expression()?)
        } else {
            None
        };
        self.consume(
            TokenType::SemiColon,
            "Expected ';' after for-loop condition.",
        )?;

        let incrementer = if !self.match_expr(&[TokenType::RightParen]) {
            Some(self.expression()?)
        } else {
            None
        };
        self.consume(TokenType::RightParen, "Expected ')' after for clause.")?;

        let mut body = self.statement()?;

        if let Some(incr) = incrementer {
            body = Stmt::Block(vec![body, Stmt::Expression(incr)]);
        };

        condition = if condition.is_none() {
            Some(Expr::Literal(Literal::True))
        } else {
            condition
        };
        body = Stmt::While {
            condition: condition.expect("Condition should exist by this point."),
            body: Box::new(body),
        };

        if let Some(init) = initializer {
            body = Stmt::Block(vec![init, body]);
        };

        Ok(body)
    }

    fn if_statement(&mut self) -> Result<Stmt, ParseError> {
        self.consume(TokenType::LeftParen, "Expected '(' after 'if'.")?;
        let condition = self.expression()?;
        self.consume(TokenType::RightParen, "Expected ')' after if condition.")?;

        let then_branch = self.statement()?;
        let else_branch = if self.match_expr(&[TokenType::Else]) {
            Some(Box::new(self.statement()?))
        } else {
            None
        };

        Ok(Stmt::If {
            condition,
            then_branch: Box::new(then_branch),
            else_branch,
        })
    }

    fn print_statement(&mut self) -> Result<Stmt, ParseError> {
        let value = self.expression()?;
        let _ = self.consume(TokenType::SemiColon, "Expect ';' after value.")?;
        Ok(Stmt::Print(value))
    }

    fn while_statement(&mut self) -> Result<Stmt, ParseError> {
        self.consume(TokenType::LeftParen, "Expect '(' after 'while'.")?;
        let condition = self.expression()?;
        self.consume(TokenType::RightParen, "Expect ')' after 'condition'.")?;
        let body = self.statement()?;
        Ok(Stmt::While {
            condition,
            body: Box::new(body),
        })
    }

    fn block_statement(&mut self) -> Result<Stmt, ParseError> {
        let mut stmts = vec![];
        while !self.check(TokenType::RightBrace) && !self.is_at_end() {
            stmts.push(self.declaration()?);
        }
        self.consume(TokenType::RightBrace, "Expected '{' after block statement.")?;
        Ok(Stmt::Block(stmts))
    }

    fn expression_statement(&mut self) -> Result<Stmt, ParseError> {
        let expr = self.expression()?;
        let _ = self.consume(TokenType::SemiColon, "Expect ';' after expression.")?;
        Ok(Stmt::Expression(expr))
    }

    fn expression(&mut self) -> Result<Expr, ParseError> {
        self.assignment()
    }

    fn assignment(&mut self) -> Result<Expr, ParseError> {
        // First we check the l-value to make sure it's a proper assignment target
        // e.g. `a`, `foo.bar`, `newPoint(x + 2).y` etc
        let expr = self.or()?;

        if self.match_expr(&[TokenType::Equal]) {
            let equals = self.previous();
            // The recursive evaluation processes the r-value as its own expression.
            let value = self.assignment()?;

            match expr {
                Expr::Variable(v) => Ok(Expr::Assignment(Assignment {
                    name: v,
                    value: Box::new(value),
                })),
                _ => Err(ParseError {
                    token: equals,
                    message: String::from("Invalid assignment target"),
                }),
            }
        } else {
            Ok(expr)
        }
    }

    fn or(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.and()?;

        while self.match_expr(&[TokenType::Or]) {
            let operator = self.previous();
            let right = self.equality()?;
            expr = Expr::Logical(Logical {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            });
        }

        Ok(expr)
    }

    fn and(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.equality()?;

        while self.match_expr(&[TokenType::And]) {
            let operator = self.previous();
            let right = self.equality()?;
            expr = Expr::Logical(Logical {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            });
        }

        Ok(expr)
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
            self.call()
        }
    }

    fn call(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.primary()?;

        loop {
            if self.match_expr(&[TokenType::LeftParen]) {
                expr = self.finish_call(expr)?;
            } else {
                break;
            }
        }

        Ok(expr)
    }

    fn finish_call(&mut self, callee: Expr) -> Result<Expr, ParseError> {
        let mut args: Vec<Expr> = vec![];
        if !self.check(TokenType::RightParen) {
            loop {
                if args.len() >= 255 {
                    return Err(ParseError {
                        token: self.peek().clone(),
                        message: "Exceeded maximum number of function arguments (255).".to_owned(),
                    });
                }
                args.push(self.expression()?);
                
                if self.match_expr(&[TokenType::Comma]) {
                    break;
                }
            }
        };
        let paren = self.consume(TokenType::RightParen, "Expected ')' after arguments.")?;

        Ok(Expr::Call(Call {
            callee: Box::new(callee),
            paren,
            args,
        }))
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
        } else if self.match_expr(&[TokenType::Identifier]) {
            // TODO: restrict keywords as idnetifiers - or maybe this will happen naturally when we implement the keywords?
            Ok(Expr::Variable(self.previous()))
        } else {
            // Exhausted all options for valid syntax - report an error
            let message = String::from("Expression expected.");
            Err(ParseError {
                token: self.peek().clone(),
                message,
            })
        }
    }

    fn parse_error(&mut self, e: ParseError) {
        if let Some(line) = e.token.line {
            println!("[{line}] Parse Error: {}", e.message);
        } else {
            println!("Parse Error: {}", e.message);
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

    /// If the next token's type is one of the passed token types, consumes that token and returns `true`.
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
    /// Does not consume the token.
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
        if self.current > 0 {
            self.tokens[self.current - 1].clone()
        } else {
            println!("Warning: Parser checked `self.previous()` from index 0");
            self.tokens[0].clone()
        }
    }
}

#[cfg(test)]
mod parser_test {
    use crate::expression::Literal;

    use super::*;

    #[test]
    fn parses_var_declaration() {
        // var a = 1;
        let tokens: Vec<Token> = vec![
            Token {
                token_type: TokenType::Var,
                lexeme: "var".to_string(),
                line: None,
                literal: None,
            },
            Token {
                token_type: TokenType::Identifier,
                lexeme: "a".to_string(),
                line: None,
                literal: None,
            },
            Token {
                token_type: TokenType::Equal,
                lexeme: "=".to_string(),
                line: None,
                literal: None,
            },
            Token {
                token_type: TokenType::Number,
                lexeme: "1".to_string(),
                line: None,
                literal: Some(Literal::Int(1)),
            },
            Token {
                token_type: TokenType::SemiColon,
                lexeme: ';'.to_string(),
                line: None,
                literal: None,
            },
            Token {
                token_type: TokenType::EOF,
                lexeme: "EOF".to_string(),
                line: None,
                literal: None,
            },
        ];

        let parsed = Parser::new(tokens).parse();
        assert_eq!(parsed.len(), 1);
        let Stmt::Var { name, initializer } = &parsed[0] else {
            panic!("expected a var declaration, got {:?}", parsed[0]);
        };

        assert_eq!(name.lexeme, "a");
        assert_eq!(initializer, &Some(Expr::Literal(Literal::Int(1))));
    }
}
