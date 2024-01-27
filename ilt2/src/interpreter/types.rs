use crate::token::TokenIdent;

use super::InterpreterValue;
use anyhow::Result;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum InterpreterTypeError {
    #[error("Invalid generic type parameters count. Expected {0} got {1}")]
    InvalidGenerics(usize, usize),
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Hash)]
pub enum InterpreterType {
    Any,
    Number,
    Int,
    Float,
    String,
    Bool,
    Array(Option<Box<InterpreterType>>),
    Tuple(Vec<InterpreterType>),
    Type,
    Void,
    Function,
    Macro,
    ToGet(TokenIdent),
}

impl InterpreterType {
    pub fn get_name(&self) -> String {
        match self {
            Self::Any => "any".to_string(),
            Self::Number => "number".to_string(),
            Self::Int => "int".to_string(),
            Self::Float => "float".to_string(),
            Self::String => "string".to_string(),
            Self::Bool => "bool".to_string(),
            Self::Array(_) => "array".to_string(),
            Self::Tuple(_) => "tuple".to_string(),
            Self::Type => "type".to_string(),
            Self::Void => "void".to_string(),
            Self::Function => "function".to_string(),
            Self::Macro => "macro".to_string(),
            Self::ToGet(ident) => format!("toget[{}]", ident.to_string()),
        }
    }

    pub fn to_string(&self) -> String {
        match self {
            Self::Any => "$any".to_string(),
            Self::Number => "$number".to_string(),
            Self::Int => "$int".to_string(),
            Self::Float => "$float".to_string(),
            Self::String => "$string".to_string(),
            Self::Bool => "$bool".to_string(),
            Self::Array(t) => match t {
                Some(t) => format!("$array[{}]", t.to_string()),
                _ => "$array".to_string(),
            },
            Self::Tuple(t) => format!("$tuple[{}]", t.iter().map(|t| t.to_string()).collect::<Vec<String>>().join(", ")),
            Self::Type => "$type".to_string(),
            Self::Void => "$void".to_string(),
            Self::Function => "$function".to_string(),
            Self::Macro => "$macro".to_string(),
            Self::ToGet(ident) => format!("$toget[{}]", ident.to_string()),
        }
    }

    pub fn with_generics(
        &self,
        generics: Option<Vec<InterpreterType>>,
    ) -> Result<InterpreterType> {
        match generics {
            Some(generics) => match self {
                Self::Array(_) => {
                    if generics.len() != 1 {
                        return Err(InterpreterTypeError::InvalidGenerics(1, generics.len()).into());
                    }
                    Ok(Self::Array(Some(Box::new(
                        generics.first().unwrap().clone(),
                    ))))
                }
                Self::Tuple(_) => Ok(Self::Tuple(generics)),
                _ => Err(InterpreterTypeError::InvalidGenerics(0, generics.len()).into()),
            },
            None => Ok(self.clone()),
        }
    }

    pub fn validate(&self, val: &InterpreterValue) -> bool {
        match self {
            InterpreterType::Any => true,
            InterpreterType::Number => val.is_number(),
            InterpreterType::Int => matches!(val, InterpreterValue::Int(_)),
            InterpreterType::Float => matches!(val, InterpreterValue::Float(_)),
            InterpreterType::String => matches!(val, InterpreterValue::String(_)),
            InterpreterType::Bool => matches!(val, InterpreterValue::Bool(_)),
            InterpreterType::Array(t) => match val {
                InterpreterValue::Array(arr) => match t {
                    Some(t) => arr.iter().all(|v| t.validate(v)),
                    _ => true,
                },
                _ => false,
            },
            InterpreterType::Tuple(t) => match val {
                InterpreterValue::Array(tuple) => {
                    if tuple.len() != t.len() {
                        return false;
                    }
                    tuple
                        .iter()
                        .zip(t.iter())
                        .all(|(v, t)| t.validate(v))
                }
                _ => false,
            },
            InterpreterType::Type => matches!(val, InterpreterValue::Type(_)),
            InterpreterType::Void => matches!(val, InterpreterValue::Void),
            InterpreterType::Function => val.is_function(),
            InterpreterType::Macro => val.is_macro(),
            InterpreterType::ToGet(ident) => {
                eprintln!("toget: {}", ident.to_string());
                false
            }
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
        InterpreterType::Array(None),
        InterpreterType::Tuple(vec![]),
        InterpreterType::Type,
        InterpreterType::Void,
        InterpreterType::Function,
        InterpreterType::Macro,
    ]
}
