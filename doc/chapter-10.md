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

UPDATE: YUP! Suddenly, the environment needs to accept a `Callable`, even though a `Literal` was fine up until now. So this is gonna be a major refactor, touching everything that accesses the `Environment` struct.

> Also, Java is able to just instantiate a class from just a plain interface apparently? So is an interface also an abstract class? Who allowed this?

Could solving this be as simple as turning a `Value` to an enum over `Literal` and `Callable`?
- Literal covers all primitives (strings, numbers, booleans, nil)
- Callable covers functions and classes
- ...that's everything right?

**OF COURSE NOT!**

Because in a binary comparison, like `get_random() > 0.5`, I don't want to compare the function, I want to compare its _result_. Do I need to rewrite the whole interpreter? Am I losing my mind?

Ok. So when evaluating an expression:
- If it's a call, compute the value then do the comparison
- If it's an identifier, look up the value then do the comparison
- If it's a literal, then it is a comparison

So we're trying to reduce everything down to comparable values, which are all literals.

New plan: Delete all the `Value`s in the interpreter - they're all just literals, so use `Literal`. The callables will evaluate to literals as well.

The environment is another story though. They need to hold literals AND uncomputed values. For another day.

---

Alright it's been a few days. I read through the rest of the chapter and I'm a bit annoyed - the functions are left unimplemented for almost the entire chapter, taking asides to mention arity and function length and a bunch of other things, before even finishing the rudimentary implementation. The function statement definition doesn't happen until close to the end. Another strong argument for a strong type system, imo -- calling a constructor on an interface makes no sense to me.

Defining the `clock` function early on creates a mess for the Rust code, because it returns a plain float rather than an AST node. Are we in the language or not?

And it gets worse. He throws a runtime exception to handle return statements. This makes a ton of sense for Java but not at all for Rust.

---

Alright, we got there in the end, but I needed to ignore the book entirely for a while.

## The Rustification

### Types

Function declarations take a "name" (the identifying token), a set of parameters, and the function body.
```rust
enum Stmt {
    // ...
    Function {
        name: Token,
        params: Vec<Token>,
        body: Vec<Stmt>,
    }
}
```

Return statements take the return keyword token, and an optional value expression.
```rust
enum Stmt {
    // ...
    Return {
        keyword: Token,
        value: Option<Box<Expr>>,
    }
}
```

A `Value` has been expanded to an enum, either a `Literal` or a reference to a `Callable`, which includes functions:
```rust
enum Value {
    Literal(Literal),
    Callable(Rc<RefCell<Callable>>),
}

enum Callable {
    Function(Function),
    Native(NativeFunction),
    Class(Class), // these come later
}

struct Function {
    pub params: Vec<Token>,
    pub body: Vec<Stmt>,
}
```

Callables also include native functions, which just take a Rust function pointer rather than a list of Lox statements:
```rust
struct NativeFunction {
    pub name: String,
    pub arity: usize,
    pub func: fn(&mut Interpreter, Vec<Value>) -> Result<Value, Interrupt<RuntimeError>>,
}
```

### Parsing

The function declaration parsing happens pretty much the same way as in the book. Same with parsing return statements and function calls. This is the point where I separated `Parser::block()` and `Parser::block_statement()`, where `block` returns a vector of statements, and `block_statement` returns a single `Stmt::Block` statement. Small but useful difference.

### Interpreting

This is the part that hurt.

#### Environment

The nested models were getting too brutal to deal with without cloning, and cloning the whole environment and all variabels just sounds like unacceptably bad performance. So I moved to reference counting with Rust's `Rc<RefCell<T>>` smart pointer. Now anyone who uses it has to borrow it first:

```rust
if let Some(env) = &self.enclosing {
    env.borrow().get(token)
    //  ^^^ necessary to access the data behind the pointer
}
```

I also removed the separate `globals` field, since globals simply act as the foundation for the base environment:
```rust
impl Interpreter {
    pub fn new() -> Self {
        let mut globals = Environment::new(None);
        let global_cell = Rc::new(RefCell::new(globals)); // could inline this, but this feels more readable
        Interpreter {
            environment: Rc::new(RefCell::new(Environment::new(Some(global_cell)))),
        }
    }
}
```

Defining a function in the new environment is similar to defining a variable, now that the `Value` enum encompasses both:
```rust
Stmt::Function { name, params, body } => {
    self.environment.borrow_mut().define(
        name,
        to_callable_value(Callable::Function(Function {
        // ^^ helper function to wrap in proper enums
            params: params.clone(),
            body: body.clone(),
        })),
    );
    None
}
```

This is the section that pushed me away from boxes and toward the `Rc/RefCell` pointer:
```rust
Stmt::Block(block) => {
    let new_env = Environment::new(Some(Rc::clone(&self.environment)));
    self.evaluate_block(block, Some(Rc::new(RefCell::new(new_env))))
}
```

#### Return handling

The entire interpreter used to bubble up a simple `Result<Value, RuntimeError>`, but the "Error" value has been changed to `Interrupt<RuntimeError>`. `Interrupt` is a new enum that encompasses both errors and return statements:

```rust
enum Interrupt<E> {
    Return { value: Value },
    Error(E),
}
```

> ☕ The book handles this by throwing the return as a runtime exception from deep in the stack, and then catching it up at the top. I hate that, and to be fair, so does the author. But it does make an icky kind of sense.

This builds in handling for early return statements by looping over the statements, and breaking when hitting a return statement.

```rust
    pub fn evaluate_block(
        &mut self,
        block: &[Stmt],
        environment: Option<Rc<RefCell<Environment>>>,
    ) -> Option<Result<Value, Interrupt<RuntimeError>>> {
        // replace previous environment with current
        let new_env = environment.unwrap_or_else(|| {
            Rc::new(RefCell::new(Environment::new(Some(Rc::clone(
                &self.environment,
            ))))) // YUCK!
        });
        let prev_env = std::mem::replace(&mut self.environment, new_env);

        let mut return_value = None;

        // evaluate contents of block
        for stmt in block {
            match self.evaluate(stmt) {
                Some(s) => match s {
                    Err(e) => match e {
                        Interrupt::Error(err) => {
                            return Some(Err(err.wrap()));
                        }
                        Interrupt::Return { value } => {
                            return_value = Some(Ok(value));
                            break;
                        }
                    },
                    Ok(_) => (),
                },
                None => (),
            }
        }

        self.environment = prev_env;

        return_value
    }
```

In a serious programming language, I imagine `RuntimeError` would also become an enum for different types of errors.

#### Native Functions

☕ The book handles native functions by simply running a java function. The environment isn't too picky about what exactly gets returned, since it's just an `Object`.

🦀 We need to extract native functions into their own member of `Callable`. You can't pass a raw function around as a value in Rust, so we make use of a function pointer, written simply as `fn()`.

```rust
pub enum Callable {
    Function(Function),
    Class(Class),
    Native(NativeFunction), // new!
}

pub struct NativeFunction {
    pub name: String,
    pub arity: usize,
    pub func: fn(&mut Interpreter, Vec<Value>) -> Result<Value, Interrupt<RuntimeError>>, // function pointer
}
```

The `name` entry isn't a token now, since there is no parsed token to refer to. Still works as the key for the environment hashmap - in fact, at some point I should probably refactor the environment methods to just take a lexeme instead of a whole token.

Define a native rust function, and the rest is straightforward.

```rust
// globals.rs
pub fn clock_native(
    _interpreter: &mut Interpreter,
    _args: Vec<Value>,
) -> Result<Value, Interrupt<RuntimeError>> {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs_f64();
    Ok(Value::Literal(Literal::Float(now as f32)))
}

// interpreter.rs
impl Interpreter {
    pub fn new() -> Self {
        let mut globals = Environment::new(None);

        globals.define_native(
            "clock", // identifier
            to_callable_value(Callable::Native(NativeFunction {
                name: "clock".into(), // identifier again.... should probably refactor this out
                arity: 0,
                func: clock_native, // the function to point to
            })),
        );

        let global_cell = Rc::new(RefCell::new(globals));

        Interpreter {
            environment: Rc::new(RefCell::new(Environment::new(Some(global_cell)))),
        }
    }
}
```

#### Calling functions

Finally, here's the call logic.

```rust
// impl Interpreter
pub fn call(
    &mut self,
    callee: Rc<RefCell<Callable>>,
    args: Vec<Value>,
    token: &Token,
) -> Result<Value, Interrupt<RuntimeError>> {
    match &*callee.borrow() { // HUH?
        Callable::Function(f) => {
            // arity check
            if f.arity() != args.len() {
                let msg = if args.len() > f.arity() {
                    "Too many arguments to function."
                } else {
                    "Too few arguments to function."
                };
                return Err(self.runtime_error(token, msg).wrap());
            };

            // build a new environment whose parent is the function's closure
            let current_env = self.environment.clone();
            let mut new_env = Environment::new(Some(self.environment.clone()));

            // bind each param to each arg
            for (param, arg) in f.params.iter().zip(args) {
                new_env.define(param, arg);
            }

            let env_cell = Rc::new(RefCell::new(new_env));

            // evaluate the function
            let result = self.evaluate_block(&f.body, Some(env_cell.clone()));

            // revert environment
            let _ = std::mem::replace(&mut self.environment, current_env);

            // catch a Return value
            match result {
                Some(Ok(val)) => Ok(val),
                Some(Err(Interrupt::Error(e))) => Err(e.wrap()),
                Some(Err(Interrupt::Return { value })) => Ok(value), // early break, not an actual error
                None => Ok(Value::Literal(Literal::Nil)), // "void" return
            }
        }
        Callable::Native(f) => {
            if args.len() != f.arity {
                return Err(self
                    .runtime_error(token, "Incorrect arity to native function")
                    .wrap());
            }
            (f.func)(self, args) // call the function
        }
        Callable::Class(c) => todo!(),
    }
}
```

After writing this, I immediately refactored the arity check into its own method. So now we just:
```rust
self.check_arity(f.arity(), args.len(), token)?;
```

Some thoughts I still have:
- This interpreter impl is getting beefy -- gonna think about how to break it out into some submodules.
- That confusing reference to a dereference: `match &*callee.borrow() {}`.
    - As I understand it, `callee.borrow()` returns a `Ref<'_, Callee>`. The match arms want a plain `Callee`, not a `Ref`. That makes sense to me.
    - But `*callee.borrow()` doesn't work when the match arms take ownership, because `Callee` doesn't and shouldn't implement `Copy`.
    - ....so why does the reference to the dereference work? Why is Rust able to reach behind the `&` reference, but not a `Ref`?
    - I found this solution from [this stackoverflow article](https://stackoverflow.com/questions/57928209/matching-with-rcrefcellt) with a similar problem.
    - **UPDATE**: I misunderstood: `Ref` isn't related to references at all. It's the *reference counting* smart pointer, and it guards the underlying data until you dereference it. This is still confusing to me, but I'm gonna keep researching and chewing on it.

Anyway, that about it does it for functions in Lox. Definitely the hardest chapter yet, required a lot of refactoring, but really got me to get more immersed into both Rust and interpreters in general. Cool stuff.
