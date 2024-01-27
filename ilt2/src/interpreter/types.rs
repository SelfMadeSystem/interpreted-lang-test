use super::InterpreterValue;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Hash)]
pub enum InterpreterType {
    Any,
    Number,
    Int,
    Float,
    String,
    Bool,
    Array,
    TypedArray(Box<InterpreterType>),
    Type,
    Void,
    Function,
    Macro,
}

impl InterpreterType {
    pub fn to_string(&self) -> String {
        match self {
            Self::Any => "any".to_string(),
            Self::Number => "number".to_string(),
            Self::Int => "int".to_string(),
            Self::Float => "float".to_string(),
            Self::String => "string".to_string(),
            Self::Bool => "bool".to_string(),
            Self::Array => "array".to_string(),
            Self::TypedArray(ty) => format!("array[{}]", ty.to_string()),
            Self::Type => "type".to_string(),
            Self::Void => "void".to_string(),
            Self::Function => "function".to_string(),
            Self::Macro => "macro".to_string(),
        }
    }

    pub fn validate(&self, val: &InterpreterValue) -> bool {
        match self  {
            InterpreterType::Any => true,
            InterpreterType::Number => val.is_number(),
            InterpreterType::Int => matches!(val, InterpreterValue::Int(_)),
            InterpreterType::Float => matches!(val, InterpreterValue::Float(_)),
            InterpreterType::String => matches!(val, InterpreterValue::String(_)),
            InterpreterType::Bool => matches!(val, InterpreterValue::Bool(_)),
            InterpreterType::Array => matches!(val, InterpreterValue::Array(_)),
            InterpreterType::TypedArray(t) => {
                if let InterpreterValue::Array(arr) = val {
                    arr.iter().all(|v| t.validate(v))
                } else {
                    false
                }
            },
            InterpreterType::Type => matches!(val, InterpreterValue::Type(_)),
            InterpreterType::Void => matches!(val, InterpreterValue::Void),
            InterpreterType::Function => val.is_function(),
            InterpreterType::Macro => val.is_macro(),
        }
    }
}

pub fn all_types() -> Vec<InterpreterType> {
    vec![
        InterpreterType::Any,
        InterpreterType::Number,
        InterpreterType::Int,
        InterpreterType::Float,
        InterpreterType::String,
        InterpreterType::Bool,
        InterpreterType::Array,
        InterpreterType::Type,
        InterpreterType::Void,
        InterpreterType::Function,
        InterpreterType::Macro,
    ]
}
