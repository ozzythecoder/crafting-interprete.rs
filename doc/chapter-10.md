# Chapter 10 - Functions
[The book](https://craftinginterpreters.com/functions.html)

To aggregate argument expression tokens in `Parser::finish_call()`, the book uses a `do-while` loop, which executes the loop once before evaluating the condition. Rust doesn't have a `do-while`, so instead we use a `loop` with a `if !condition break` at the end.

```rust
loop {
    if args.len() >= 255 {
        return Err(ParseError {
            token: self.peek().clone(),
            message: "Exceeded maximum number of function arguments (255).".to_owned(),
        });
    }
    args.push(self.expression()?);
    
    // the "while" part
    if self.match_expr(&[TokenType::Comma]) {
        break;
    }
}
```