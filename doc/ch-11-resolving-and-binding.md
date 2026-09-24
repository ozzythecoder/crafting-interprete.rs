# Chapter 11. Resolving and Binding

[The book](https://craftinginterpreters.com/resolving-and-binding.html)

## Resolver struct

☕ The book is using a Stack for this data structure, which seems to just be a vector with some handy APIs for FILO operations.
🦀 Gonna start out with a `Vec` and see what happens. Also, similar to how we did the environments, I'm going to stick with an `Rc<RefCell<Vec<HashMap<String, bool>>>>`. Say that ten times fast.

Already unsure how to handle errors here -- the book is calling `Lox.error`, but in rust we need something to propagate. For now I'm pushing to a `vec` at `self.errors`.

### Scope & locals

☕ Java uses the entire `Expr` object as a key for the `locals` hash map. Kinda deranged imo.

🦀 Decided to just do an ID sequence to keep it simple.

- We're bringing back the `Interpreter::globals` field I guess...
- ~~We're also rebuilding the environment to add a specifier for the *distance* from the current scope.~~ Actually we're just putting that straight on the resolver itself. Recursing into `self` just became too much.

### Reflections

Got to learn a lot about Rust's `Box`, `Rc` and `RefCell` this time around. I'm a little wary of the last two, since they move the borrow check from compile time to runtime -- trying to do simultaneous mutable borrows causes a panic, rather than a compiler error. I can see how that could swallow some errors too early. But the borrow checker is very conservative, and it seems important to learn when and how to relax it a bit.

The need for a refactor grows... I'm still uncertain of the best Rust idioms for this. Feels like I'm kinda forcing the object-oriented model where it doesn't fit, with every struct impl having a `new() -> Self` function. Maybe that's ok for now, but the cross-cutting concerns are getting a little freaky, with the interpreter now, in theory, holding methods that should belong to the environment or the resolver.

But this is functional for now, raises the proper errors, and that feels pretty good.

