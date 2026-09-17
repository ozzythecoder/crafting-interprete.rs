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

> [!NOTE]
> New grammar! The highest-precedence expression is now an assignment.
> `expression  -> assignment ;`
> `assignment  -> IDENTIFIER "=" assignment |  equality ;`

When parsing variables, we run into a different situation that is harder for a recursive descent parser. We only have one token of lookahead, so we need to split on the '=' in the variable assignment. We evaluate the right-hand side as a valid expression, _then_ we evaluate the left-hand side as a valid assignment target. Supposedly, this allows us to do complex assignments like `newPoint(x + 2, 0).y = 3`, where we call a function and update its property at once.

- 🦀 Added a unit test to the parser to check for variable assignments. Will prob expand as our grammar gets more complex.

## Environments

Captures variable state. Implemented with a `HashMap<String, Literal>`. To implement **scope**, it looks like we'll be nesting environments in one another.

Update: I was right! Pretty simple to conceptualize block evaluation:

1. instantiate a new environment, setting the previous environment to enclose the new one
2. hold the previous environment in a temporary variable
3. execute the block commands
4. restore the previous environment

This was tricky to do in Rust just because of move semantics. Both setting a new environment and holding the old environment temporarily required taking ownership of the environment, which is impossible from behind a mutable reference.

```rust
Stmt::Block(block) => {
    let this_env = Environment::new(Some(self.environment)); // error: cannot move
    let prev_env = self.environment; // error: cannot move
    self.environment = this_env;

    // evaluate block contents
    let result = self.interpret(block);
    
    // restore previous env
    self.environment = prev_env;

    // ...
}
```

The solution was to use `std::mem::take`, which sets the passed variable to its default state. This does require moving values around, but at least doesn't require implementing `Clone` for each block execution.

```rust
Stmt::Block(b) => {
    let prev_env = std::mem::take(&mut self.environment); // 👍 works; self.environment is now Environment::default()
    self.environment = Environment::new(Some(prev_env)); // instantiate new environment with `prev_env` as its parent scope

    // evaluate block contents
    let result = self.interpret(block);
    
    // take enclosing environment pointer back from child
    let child_env = std::mem::take(&mut self.environment);
    // dereferences the child environment from the pointer, gets its enclosing environment, and sets the current environment back.
    // this panics if there is no enclosing block, since we shouldn't even be here otherwise.
    self.environment = *child_env
        .enclosing
        .expect("Block env cannot be build without a parent");
}
```

### Vocabulary

- _l-value_ - the left-hand side of an assignment expression
- _r-value_ - the expression, or right-hand side of an assignment expression
