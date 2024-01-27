use std::rc::Rc;

use anyhow::Error;

use crate::ast::{AstNode, AstNodeType};

use super::{types::InterpreterType, InterpreterError, NativeFn, NativeMacro};

#[derive(Debug, Clone)]
pub enum InterpreterValue {
    Int(i64),
    Float(f64),
    String(String),
    Bool(bool),
    Array(Vec<Rc<InterpreterValue>>),
    Type(InterpreterType),
    Void,
    Function {
        name: String,
        generics: Option<Vec<String>>,
        params: Vec<(String, InterpreterType)>,
        return_type: InterpreterType,
        body: Vec<AstNode>,
    },
    NativeFunction {
        name: String,
        body: NativeFn,
    },
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
            Self::Array(_) => InterpreterType::Array(None),
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
                a.iter()
                    .map(|v| v.to_formatted_string())
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
                Ok(Self::Array(array))
            }
            _ => Err(
                InterpreterError::InvalidConstValue(value.clone(), value.line, value.col).into(),
            ),
        }
    }
}
