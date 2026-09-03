# Ch 8. Statements and State
**[The Chapter in The Book](https://craftinginterpreters.com/statements-and-state.html)**

We're adding support for print statements and expression statements. Once again, instead of AST classes and codegen, we're relying on Rust's enum system.

```rust
#[derive(Debug, Clone)]
pub enum Stmt {
    Expression(Expr),
    Print(Expr),
}
```

In the parser, we now return a vector of `Stmt`s rather than a single `Expr`. And in the interpreter, we evaluate each `Stmt`, printing the print statements and (for now) discarding the expression statements. We've also changed the interpreter to a struct and impl block to maintain internal state, rather than bare functions. So the new call in `Lox::main` is now:
```rust
match Interpreter::new(ast).parse { // etc...
```