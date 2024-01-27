use super::InterpreterValue;
use std::rc::Rc;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Hash)]
pub struct InterpreterType {
    pub name: String,
    // pub generics: Vec<InterpreterType>,
    pub is_macro: bool,
}

macro_rules! ttt {
    ($name:literal) => {
        (
            $name.to_string(),
            InterpreterType {
                name: $name.to_string(),
                is_macro: false,
            },
        )
    };
}

pub fn all_types() -> Vec<(String, InterpreterType)> {
    vec![
        ttt!("any"),
        ttt!("number"),
        ttt!("int"),
        ttt!("float"),
        ttt!("string"),
        ttt!("bool"),
        ttt!("array"),
        ttt!("type"),
        ttt!("void"),
        ttt!("function"),
        ttt!("macro"),
    ]
}
