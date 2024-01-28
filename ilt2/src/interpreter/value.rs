use std::{cell::RefCell, collections::{HashMap, HashSet}, rc::Rc};

use anyhow::{Error, Result};

use crate::{
    ast::{AstNode, AstNodeType},
    token::TokenIdent,
};

use super::{types::InterpreterType, InterpreterError, NativeFn, NativeMacro};

#[derive(Debug, Clone, PartialEq)]
pub enum InterpreterValue {
    Int(i64),
    Float(f64),
    String(String),
    Bool(bool),
    Array(RefCell<Vec<Rc<InterpreterValue>>>),
    Dict(RefCell<HashMap<String, Rc<InterpreterValue>>>),
    Type(InterpreterType),
    Void,
    Function {
        name: String,
        generics: Option<Vec<(String, Option<TokenIdent>)>>,
        params: Vec<(String, InterpreterType)>,
        return_type: InterpreterType,
        body: Vec<AstNode>,
    },
    NativeFunction {
        name: String,
        body: NativeFn,
    },
    #[allow(dead_code)]
    Macro {
        name: String,
        params: Vec<String>,
        body: Vec<AstNode>,
    },
    NativeMacro {
        name: String,
        body: NativeMacro,
    },
    // TODO: Scope, AstNode for macros
}

impl InterpreterValue {
    pub fn get_type(&self) -> InterpreterType {
        match self {
            Self::Int(_) => InterpreterType::Int,
            Self::Float(_) => InterpreterType::Float,
            Self::String(_) => InterpreterType::String,
            Self::Bool(_) => InterpreterType::Bool,
            Self::Array(vals) => {
                InterpreterType::Tuple(vals.borrow().iter().map(|v| v.get_type()).collect())
            }
            Self::Dict(dict) => InterpreterType::Dict(Box::new({
                // union of all values
                let mut tys = HashSet::new();
                for (_, val) in dict.borrow().iter() {
                    tys.insert(val.get_type());
                }
                if tys.len() == 1 {
                    tys.into_iter().next().unwrap()
                } else {
                    InterpreterType::Union(tys.into_iter().collect())
                }
            })),
            Self::Type(_) => InterpreterType::Type,
            Self::Void => InterpreterType::Void,
            Self::Function { .. } => InterpreterType::Function,
            Self::NativeFunction { .. } => InterpreterType::Function,
            Self::Macro { .. } => InterpreterType::Macro,
            Self::NativeMacro { .. } => InterpreterType::Macro,
        }
    }

    pub fn check_type(&self, ty: &InterpreterType) -> bool {
        ty.validate(self)
    }

    pub fn is_number(&self) -> bool {
        match self {
            Self::Int(_) | Self::Float(_) => true,
            _ => false,
        }
    }

    pub fn is_function(&self) -> bool {
        match self {
            Self::Function { .. } | Self::NativeFunction { .. } => true,
            _ => false,
        }
    }

    pub fn is_macro(&self) -> bool {
        match self {
            Self::Macro { .. } | Self::NativeMacro { .. } => true,
            _ => false,
        }
    }

    pub fn as_type(&self, ty: &InterpreterType) -> Result<Self> {
        match ty {
            InterpreterType::Any => Ok(self.clone()),
            InterpreterType::ToGet(t) => {
                eprintln!("Warning: using ToGet type {:?}", t);
                Ok(self.clone())
            }
            InterpreterType::Number => match self {
                Self::Int(i) => Ok(Self::Int(*i)),
                Self::Float(f) => Ok(Self::Float(*f)),
                Self::String(s) => Ok(Self::Float(s.parse::<f64>()?)),
                _ => Err(InterpreterError::InvalidTypeCast(
                    self.get_type().to_string(),
                    ty.to_string(),
                )
                .into()),
            },
            InterpreterType::Int => match self {
                Self::Int(i) => Ok(Self::Int(*i)),
                Self::Float(f) => Ok(Self::Int(*f as i64)),
                Self::String(s) => Ok(Self::Int(s.parse::<i64>()?)),
                _ => Err(InterpreterError::InvalidTypeCast(
                    self.get_type().to_string(),
                    ty.to_string(),
                )
                .into()),
            },
            InterpreterType::Float => match self {
                Self::Int(i) => Ok(Self::Float(*i as f64)),
                Self::Float(f) => Ok(Self::Float(*f)),
                Self::String(s) => Ok(Self::Float(s.parse::<f64>()?)),
                _ => Err(InterpreterError::InvalidTypeCast(
                    self.get_type().to_string(),
                    ty.to_string(),
                )
                .into()),
            },
            InterpreterType::String => Ok(Self::String(self.to_string())),
            InterpreterType::Bool => match self {
                Self::Bool(b) => Ok(Self::Bool(*b)),
                _ => Err(InterpreterError::InvalidTypeCast(
                    self.get_type().to_string(),
                    ty.to_string(),
                )
                .into()),
            },
            InterpreterType::Array(aty) => match self {
                Self::Array(vals) => {
                    if let Some(aty) = aty {
                        let mut new_vals = Vec::new();
                        for val in vals.borrow().iter() {
                            new_vals.push(Rc::new(val.as_type(aty)?));
                        }
                        Ok(Self::Array(RefCell::new(new_vals)))
                    } else {
                        Ok(self.clone())
                    }
                }
                _ => Err(InterpreterError::InvalidTypeCast(
                    self.get_type().to_string(),
                    ty.to_string(),
                )
                .into()),
            },
            InterpreterType::Tuple(tys) => match self {
                Self::Array(vals) => {
                    if vals.borrow().len() != tys.len() {
                        return Err(InterpreterError::InvalidTypeCast(
                            self.get_type().to_string(),
                            ty.to_string(),
                        )
                        .into());
                    }
                    let mut new_vals = Vec::new();
                    for (val, ty) in vals.borrow().iter().zip(tys.iter()) {
                        new_vals.push(Rc::new(val.as_type(ty)?));
                    }
                    Ok(Self::Array(RefCell::new(new_vals)))
                }
                _ => Err(InterpreterError::InvalidTypeCast(
                    self.get_type().to_string(),
                    ty.to_string(),
                )
                .into()),
            },
            InterpreterType::Union(tys) => {
                for ty in tys.iter() {
                    if ty.validate(self) {
                        return Ok(self.clone());
                    }
                }
                Err(
                    InterpreterError::InvalidTypeCast(self.get_type().to_string(), ty.to_string())
                        .into(),
                )
            }
            InterpreterType::Dict(ty) => match self {
                Self::Dict(dict) => {
                    let mut new_dict = HashMap::new();
                    for (key, val) in dict.borrow().iter() {
                        new_dict.insert(key.clone(), Rc::new(val.as_type(ty)?));
                    }
                    Ok(Self::Dict(RefCell::new(new_dict)))
                }
                _ => Err(InterpreterError::InvalidTypeCast(
                    self.get_type().to_string(),
                    ty.to_string(),
                )
                .into()),
            },
            InterpreterType::Type => match self {
                Self::Type(ty) => Ok(Self::Type(ty.clone())),
                _ => Err(InterpreterError::InvalidTypeCast(
                    self.get_type().to_string(),
                    ty.to_string(),
                )
                .into()),
            },
            InterpreterType::Void => match self {
                Self::Void => Ok(Self::Void),
                _ => Err(InterpreterError::InvalidTypeCast(
                    self.get_type().to_string(),
                    ty.to_string(),
                )
                .into()),
            },
            InterpreterType::Function => match self {
                Self::Function { .. } | Self::NativeFunction { .. } => Ok(self.clone()),
                _ => Err(InterpreterError::InvalidTypeCast(
                    self.get_type().to_string(),
                    ty.to_string(),
                )
                .into()),
            },
            InterpreterType::Macro => match self {
                Self::Macro { .. } | Self::NativeMacro { .. } => Ok(self.clone()),
                _ => Err(InterpreterError::InvalidTypeCast(
                    self.get_type().to_string(),
                    ty.to_string(),
                )
                .into()),
            },
        }
    }

    pub fn to_formatted_string(&self) -> String {
        match self {
            Self::String(s) => format!("\"{}\"", s),
            _ => self.to_string(),
        }
    }

    pub fn to_string(&self) -> String {
        match self {
            Self::Int(i) => i.to_string(),
            Self::Float(f) => f.to_string(),
            Self::String(s) => s.to_string(),
            Self::Bool(b) => b.to_string(),
            Self::Array(a) => format!(
                "[{}]",
                a.borrow()
                    .iter()
                    .map(|v| v.to_formatted_string())
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            Self::Dict(m) => format!(
                "{{{}}}",
                m.borrow()
                    .iter()
                    .map(|(k, v)| format!("{}: {}", k, v.to_formatted_string()))
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            Self::Type(ty) => ty.to_string(),
            Self::Void => "Void".to_string(),
            Self::Function { name, params, .. } => {
                format!("Function {{ name: {}, params: {:?} }}", name, params)
            }
            Self::NativeFunction { name, .. } => format!("NativeFunction {{ name: {} }}", name),
            Self::Macro { name, params, .. } => {
                format!("Macro {{ name: {}, params: {:?} }}", name, params)
            }
            Self::NativeMacro { name, .. } => format!("NativeMacro {{ name: {} }}", name),
        }
    }
}

impl TryFrom<AstNode> for InterpreterValue {
    type Error = Error;

    fn try_from(value: AstNode) -> Result<Self, Self::Error> {
        match value.ty {
            AstNodeType::Int(value) => Ok(Self::Int(value)),
            AstNodeType::Float(value) => Ok(Self::Float(value)),
            AstNodeType::String(value) => Ok(Self::String(value)),
            AstNodeType::Array(value) => {
                let mut array = Vec::new();
                for value in value.iter() {
                    array.push(Rc::new((value.clone()).try_into()?));
                }
                Ok(Self::Array(RefCell::new(array)))
            }
            _ => Err(
                InterpreterError::InvalidConstValue(value.clone(), value.line, value.col).into(),
            ),
        }
    }
}
