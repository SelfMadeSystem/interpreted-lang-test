use std::collections::HashMap;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Hash)]
pub struct InterpreterType {
    pub name: &'static str,
    // pub generics: Vec<InterpreterType>,
    pub is_macro: bool,
}

impl InterpreterType {
    pub fn to_string(&self) -> String {
        self.name.to_string()
    }
}

macro_rules! define_types {
    ($(($name:ident, $upper:ident)),* $(,)?) => {
        impl InterpreterType {
            $(
                #[allow(dead_code)]
                pub const $upper: Self = Self {
                    name: stringify!($name),
                    is_macro: false,
                };
            )*
        }

        pub fn all_types() -> HashMap<String, InterpreterType> {
            let mut map = HashMap::new();
            $(
                map.insert(stringify!($name).to_string(), InterpreterType {
                    name: stringify!($name),
                    is_macro: false,
                });
            )*
            map
        }
    };
}

define_types! {
    (any, ANY),
    (number, NUMBER),
    (int, INT),
    (float, FLOAT),
    (string, STRING),
    (bool, BOOL),
    (array, ARRAY),
    (type, TYPE),
    (void, VOID),
    (function, FUNCTION),
    (macro, MACRO),
}
