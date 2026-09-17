# Chapter 9. Control Flow
[The book](https://craftinginterpreters.com/control-flow.html)

## If/then/else

Working on control flow, I ran into an `isTruthy()` function that I hadn't implemented in the Rust interpreter yet. I tried a few clunky versions of copying the book's approach, before realizing I could just create the method on the `Value` itself:

```rust
impl Value {
    fn is_truthy(&self) -> bool {
        match self.0 {
            Literal::False | Literal::Nil => false,
            Literal::Boolean(b) => b,
            _ => true,
        }
    }
}
```

And then when evaluating conditionals, it's much easier to grab the internal value:
```rust
match self.evaluate_expression(condition) {
    Ok(r) => {
        if r.is_truthy() {
            // ...
```

## For/while loops

Turned off my brain for a bit here. I naïvely transliterated the Java code to Rust, which caused this nasty infinite loop:
```rust
Stmt::While { condition, body } => match self.evaluate_expression(condition) => {
    Ok(val) => {
        while val.is_truthy() {
            self.evaluate(body)?
        }
        // ...
    }
}
```

So `condition.is_truthy()` is evaluated once, and then gets stuck. And I was wondering why the debugger kept crashing my computer...

The updated code uses a plain `loop` and runs `evaluate_expression` repeatedly, breaking on a falsy value or an error:
```rust
Stmt::While { condition, body } => {
    loop {
        match self.evaluate_expression(condition) => {
            Ok(val) => {
                if !val.is_truthy() {
                    break;
                }
                self.evaluate(body)?;
            },
            Err(e) => return Some(Err(e)),
        }
    }
    None
}
```