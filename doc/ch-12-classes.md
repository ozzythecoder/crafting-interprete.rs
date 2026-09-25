# Chapter 12. Classes

[The book](https://craftinginterpreters.com/classes.html)

## Classes

Right off the bat, we're back to enums. Rather than creating a `LoxClass`, we're going to extend the `Value::Callable` enum to include a `Callable::Class`.

Nothing too bad so far. Caught a nasty bug in the runtime -- when providing an interpreter to the resolver, I didn't realize I should be using the *same* interpreter for running the program. Sounds obvious now that I say it though.

```rust
resolver.resolve(&ast);
match resolver.into_interpreter().interpret(&ast) {
    // ...
}
```
