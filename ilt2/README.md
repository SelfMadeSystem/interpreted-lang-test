# Interpreted Lang Test 2

This is my second attempt at an interpreted language. It's implemented in Rust
and has a much more Lisp-like syntax.

## Usage

```bash
cargo run ./test.ilt2
```

## Syntax

The syntax has prefix notation. Honestly just look at the example `.ilt2`
files. I just wanted it to be easy to implement, and postfix/prefix notation is
very easy to implement.

Functions are first-class, and can be passed around as arguments.
