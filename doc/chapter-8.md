# Ch 8. Statements and State
**[The Chapter in The Book](https://craftinginterpreters.com/statements-and-state.html)**

We're adding support for print statements and expression statements. Once again, instead of the AST classes and codegen in the Java version, we're relying on Rust's enum system.

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

>[!NOTE]
> New grammar! The highest-precedence expression is now an assignment.
> `expression  -> assignment ;`
> `assignment  -> IDENTIFIER "=" assignment |  equality ;`

When parsing variables, we run into a different situation that is harder for a recursive descent parser. We only have one token of lookahead, so we need to split on the '=' in the variable assignment. We evaluate the right-hand side as a valid expression, *then* we evaluate the left-hand side as a valid assignment target. Supposedly, this allows us to do complex assignments like `newPoint(x + 2, 0).y = 3`, where we call a function and update its property at once.

- 🦀 Added a unit test to the parser to check for variable assignments. Will prob expand as our grammar gets more complex.

### Vocabulary
- *l-value* - the left-hand side of an assignment expression
- *r-value* - the expression, or right-hand side of an assignment expression