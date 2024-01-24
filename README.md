# Interpreted Lang Test

This is a test of an interpreted language. It has lisp-like syntax, and is
implemented in Rust.

## Usage

```bash
cargo run ./test.thing
```

## Syntax

The syntax has prefix notation. Honestly just look at [test.thing](./test.thing)
for an example.

Functions are first-class, and can be passed around as arguments.

If statements and while loops aren't technically implemented, but they've been
implemented using native functions. This is why you have to wrap the condition
and the body in an anonymous function.
