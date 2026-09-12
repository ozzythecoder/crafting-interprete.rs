# Chapter 10 - Functions

[The book](https://craftinginterpreters.com/functions.html)

## Loops

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

## Callables

This one's gonna suck huh.

- ☕ The book handles interpreting functions by type-casting the callee into a `LoxCallable`, which is an object with a `call` method that takes `this` and an array of arguments as parameters.
- [x] 🦀 We don't have plain "objects", and type-casting from a `Value(Literal)` to a `Callable` struct isn't doable outright. We can solve that by just creating a new `Callable` struct.
- [ ] However, the issue is that we are now moving the `Result<Stmt, RuntimeError>` chain OUTSIDE of the interpreter struct, which feels icky. How do we solve this?

### Call type errors

- ☕ In case a non-function is called, the book is catching a JVM exception, and re-throwing a new error, stating that only classes and functions can be called. This is checked with an `instanceof` check.
- 🦀 We don't do exceptions, and we don't have a literal `instanceof`. We're checking a `Value`, and we want to see if it is a valid callable.
- [x] What's the Rusty way to deal with this?
    - [x] ~~Perhaps a method on the `Value` impl?~~ No, there's no field to check for.
    - [x] ~~We'll do a `Callable` trait, and an unfulfilled `impl Callable for Value` for now. Will return to this and see if it works.~~ Doesn't work - callable needs to be its own thing.

It's starting to feel like the `Value` struct is starting to fail as a replacement for a Java `Object`.

YUP! Suddenly, the environment needs to accept a `Callable`, even though a `Literal` was fine up until now. So this is gonna be a major refactor, touching everything that accesses the `Environment` struct.

> Also, Java is able to just instantiate a class from just a plain interface apparently? So is an interface also an abstract class? Who allowed this?

- [ ] Could solving this be as simple as turning a `Value` to an enum over `Literal` and `Callable`?
    - Literal covers all primitives (strings, numbers, booleans, nil)
    - Callable covers functions and classes
    - ...that's everything right?
