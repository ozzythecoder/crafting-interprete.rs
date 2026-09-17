# Ch 7. Evaluating Expressions
- [The Book](https://craftinginterpreters.com/evaluating-expressions.html)

## Representing Values & Evaluating Expressions

- ☕ To represent all Lox values, Nystrom uses `java.lang.Object`, and narrows based on `instanceof`.
    - He also recommends the visitor pattern from the AST printer example.
- 🦀 We will probably be using enums again.
    - We never implemented the visitor pattern in the Rust AST printer, opting instead for a `match` statement on the different expression categories. We're gonna do that again too.

### Changes

- Created new `Value(Literal)` struct to store expression results
- Created new `evaluate()` function
    - Performs the operation depending on expression types (matching on `Literal::<type>`),
    - Handles errors gracefully
    - So far, this is the only part of the interpreter that is much more verbose in Rust than in Java
- Skipping the `interpret` function
    - ☕ This was used to catch the exception thrown from deep inside the tree.
    - 🦀 Rust doesn't support exceptions, so instead we manually bubble up the error by returning a `Result` from `evaluate`.
- Skipping the `stringify` function
    - ☕ This is where the Java program defines how to print the strings to the console
    - 🦀 We've already defined this by deriving `Debug` and implementing `ToString`.
- Wiring everything together in the `Lox` impl
    - `Lox::run` constructs its own scanner and parser, and then evaluates the final expr.
    - Set up the REPL
- Breaking out everything into more discrete modules