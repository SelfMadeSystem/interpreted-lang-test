use crate::token::TokenIdent;

use super::InterpreterValue;
use anyhow::Result;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum InterpreterTypeError {
    #[error("Invalid generic type parameters count. Expected {0} got {1}")]
    InvalidGenerics(usize, usize),
    #[error("Don't use $struct[...] directly. To create a struct type, use the @struct macro")]
    DontUseStruct,
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
    Union(Vec<InterpreterType>),
    /// dict key is always string
    Dict(Box<InterpreterType>),
    // same as dict, but key-value pairs are fixed
    Struct(Vec<(String, InterpreterType)>),
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
            Self::Union(_) => "union".to_string(),
            Self::Dict(_) => "dict".to_string(),
            Self::Struct(_) => "struct".to_string(),
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
            Self::Tuple(t) => format!(
                "$tuple[{}]",
                t.iter()
                    .map(|t| t.to_string())
                    .collect::<Vec<String>>()
                    .join(", ")
            ),
            Self::Union(t) => format!(
                "$union[{}]",
                t.iter()
                    .map(|t| t.to_string())
                    .collect::<Vec<String>>()
                    .join(", ")
            ),
            Self::Dict(t) => format!("$dict[{}]", t.to_string()),
            Self::Struct(t) => format!(
                "$struct[{}]",
                t.iter()
                    .map(|(k, v)| format!("{}: {}", k.to_string(), v.to_string()))
                    .collect::<Vec<String>>()
                    .join(", ")
            ),
            Self::Type => "$type".to_string(),
            Self::Void => "$void".to_string(),
            Self::Function => "$function".to_string(),
            Self::Macro => "$macro".to_string(),
            Self::ToGet(ident) => format!("$toget[{}]", ident.to_string()),
        }
    }

    pub fn with_generics(&self, generics: Option<Vec<InterpreterType>>) -> Result<InterpreterType> {
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
                Self::Union(_) => Ok(Self::Union(generics)),
                Self::Dict(_) => {
                    if generics.len() != 1 {
                        return Err(InterpreterTypeError::InvalidGenerics(1, generics.len()).into());
                    }
                    Ok(Self::Dict(Box::new(generics.first().unwrap().clone())))
                }
                Self::Struct(_) => Err(InterpreterTypeError::DontUseStruct.into()),
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
                    Some(t) => arr.borrow().iter().all(|v| t.validate(v)),
                    _ => true,
                },
                _ => false,
            },
            InterpreterType::Tuple(t) => match val {
                InterpreterValue::Array(tuple) => {
                    let tuple = tuple.borrow();
                    if tuple.len() != t.len() {
                        return false;
                    }
                    tuple.iter().zip(t.iter()).all(|(v, t)| t.validate(v))
                }
                _ => false,
            },
            InterpreterType::Union(t) => t.iter().any(|t| t.validate(val)),
            InterpreterType::Dict(t) => match val {
                InterpreterValue::Dict(dict) => {
                    let dict = dict.borrow();
                    dict.iter().all(|(_, v)| t.validate(v))
                }
                _ => false,
            },
            InterpreterType::Struct(t) => match val {
                InterpreterValue::Dict(dict) => {
                    let dict = dict.borrow();
                    dict.iter().all(|(k, v)| {
                        t.iter()
                            .find(|(k1, _)| k == k1)
                            .map(|(_, t)| t.validate(v))
                            .unwrap_or(false)
                    })
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

    /// Whether ty can be assigned to self
    ///
    /// e.g. `int` is assignable to `number`, but `number` is not assignable to `int`
    ///
    /// basically, true if self is the same or more general than ty
    pub fn is_assignable(&self, ty: &InterpreterType) -> bool {
        if let InterpreterType::ToGet(t) = self {
            eprintln!("toget: {}", t.to_string());
            return false;
        }
        if self == ty {
            return true;
        }
        match (self, ty) {
            (InterpreterType::Any, _) => true,
            (InterpreterType::Number, InterpreterType::Int) => true,
            (InterpreterType::Number, InterpreterType::Float) => true,
            (
                InterpreterType::Array(None),
                InterpreterType::Array(_) | InterpreterType::Tuple(_),
            ) => true,
            (InterpreterType::Array(Some(t)), InterpreterType::Array(Some(ty))) => {
                t.is_assignable(ty)
            }
            (InterpreterType::Array(Some(t)), InterpreterType::Tuple(ty)) => {
                ty.iter().all(|ty| t.is_assignable(ty))
            }
            (InterpreterType::Tuple(t), InterpreterType::Tuple(ty)) => {
                t.len() == ty.len() && t.iter().zip(ty.iter()).all(|(t, ty)| t.is_assignable(ty))
            }
            (InterpreterType::Union(t), InterpreterType::Void) if t.len() == 0 => true, // void and union of nothing are the same
            (InterpreterType::Union(t), t1) if t.len() == 1 => t[0].is_assignable(t1), // union of one is the same as the type
            (InterpreterType::Union(t), _) => t.iter().any(|t| t.is_assignable(ty)),
            (InterpreterType::Dict(t), InterpreterType::Dict(ty)) => t.is_assignable(ty),
            (InterpreterType::Type, InterpreterType::Type) => true,
            (InterpreterType::Void, InterpreterType::Void) => true,
            (InterpreterType::Function, InterpreterType::Function) => true,
            (InterpreterType::Macro, InterpreterType::Macro) => true,
            _ => false,
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
        InterpreterType::Union(vec![]),
        InterpreterType::Dict(Box::new(InterpreterType::Any)),
        InterpreterType::Struct(vec![]),
        InterpreterType::Type,
        InterpreterType::Void,
        InterpreterType::Function,
        InterpreterType::Macro,
    ]
}
