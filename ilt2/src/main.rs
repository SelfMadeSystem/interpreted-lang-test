use default_fns::{native_functions, native_macros};
use interpreter::interpret;
use lexer::Lexer;
use parser::Parser;
use std::env;
use std::fs;
use std::io::{self, Read};

mod ast;
mod default_fns;
mod interpreter;
mod lexer;
mod parser;
mod token;

fn main() {
    let args: Vec<String> = env::args().collect();
    let input = if args.len() > 1 {
        // Read from file if argument is provided
        fs::read_to_string(&args[1]).expect("Failed to read file")
    } else {
        // Otherwise, read from stdin
        let mut buffer = String::new();
        io::stdin()
            .read_to_string(&mut buffer)
            .expect("Failed to read from stdin");
        buffer
    };

    let lexer = Lexer::new(&input);
    let mut parser = Parser::try_new(lexer).expect("Failed to create parser");
    let ast = parser.parse().expect("Failed to parse AST");

    let result =
        interpret(ast, native_functions(), native_macros()).expect("Failed to interpret AST");

    println!("result: {:#?}", result);
}
