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

