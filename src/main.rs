use lexer::Lexer;

use crate::default_fns::default_native_functions;
use crate::{interpreter::interpret, parser::Parser};

mod default_fns;
mod ast;
mod interpreter;
mod lexer;
mod parser;
mod token;
use std::env;
use std::fs;
use std::io::{self, Read};

fn main() {
    let args: Vec<String> = env::args().collect();
    let input = if args.len() > 1 {
        // Read from file if argument is provided
        fs::read_to_string(&args[1]).expect("Failed to read file")
    } else {
        // Otherwise, read from stdin
        let mut buffer = String::new();
        io::stdin().read_to_string(&mut buffer).expect("Failed to read from stdin");
        buffer
    };

    let lexer = Lexer::new(&input);
    let mut parser = Parser::try_new(lexer).expect("Failed to create parser");
    let ast = parser.parse().expect("Failed to parse AST");

    let result = interpret(ast, default_native_functions()).expect("Failed to interpret AST");

    println!("result: {:#?}", result);
}
