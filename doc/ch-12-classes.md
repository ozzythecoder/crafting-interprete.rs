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

### Instances

☕ The book implements classes by... `implement`ing them. `class LoxClass implements LoxCallable`.

🦀 We do the same just by building out a new enum member: `Callable(Class)`. I've had this stubbed out for a while with a `todo!()` marker, so it's just about filling that in now.

So far so good, but the next wrinkle is instances. `class LoxInstance` doesn't map neatly onto any of our existing structures.

My hypothesis: add another entry to the enum: `Value::ClassInstance`.

Works! So far. Ran into another issue: shared references in `Interpreter::call()`. To get owned data, I could do `c.clone()`, but then I'd be doubling the amount of memory used for every class. So I think ClassInstance will need to hold an ~~`Rc<RefCell<Class>>`~~ -- actually, I can just use an `Rc<Class>` for this. The `RefCell` is for mutability, but the classes are immutable once defined.

