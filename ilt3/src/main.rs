use lexer::Lexer;
use std::env;
use std::fs;
use std::io::{self, Read};

use crate::parser::Parser;
use crate::runtime::Runtime;

mod builtin_functions;
mod ir;
mod lexer;
mod parser;
mod runtime;
mod scope;
mod value;

fn main() {
    let args: Vec<String> = env::args().collect();
    let input = if args.len() > 1 {
        // Read from file if argument is provided
        fs::read_to_string(&args[1]).expect("Failed to read file")
    } else {
        fs::read_to_string("./test.ilt3").expect("Failed to read file")
        // Otherwise, read from stdin
        // let mut buffer = String::new();
        // io::stdin()
        //     .read_to_string(&mut buffer)
        //     .expect("Failed to read from stdin");
        // buffer
    };

    let mut lexer = Lexer::new(&input);
    let lines = lexer.parse().expect("Failed to lex input");
    let mut parser = Parser::new(&lines);
    let ast = parser.parse().expect("Failed to parse input");
    let mut runtime = Runtime::from_ast(ast);
    runtime.add_builtin_functions();
    let result = runtime.run().expect("Failed to run program");
    println!("{:#?}", result);
}
