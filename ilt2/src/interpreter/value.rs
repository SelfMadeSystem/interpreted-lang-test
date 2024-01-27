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
            Self::Int(_) => InterpreterType {
                name: "int",
                is_macro: false,
            },
            Self::Float(_) => InterpreterType {
                name: "float",
                is_macro: false,
            },
            Self::String(_) => InterpreterType {
                name: "string",
                is_macro: false,
            },
            Self::Bool(_) => InterpreterType {
                name: "bool",
                is_macro: false,
            },
            Self::Array(_) => InterpreterType {
                name: "array",
                is_macro: false,
            },
            Self::Type(_) => InterpreterType {
                name: "type",
                is_macro: false,
            },
            Self::Void => InterpreterType {
                name: "void",
                is_macro: false,
            },
            Self::Function { .. } => InterpreterType {
                name: "function",
                is_macro: false,
            },
            Self::NativeFunction { .. } => InterpreterType {
                name: "function",
                is_macro: false,
            },
            Self::Macro { .. } => InterpreterType {
                name: "macro",
                is_macro: false,
            },
            Self::NativeMacro { .. } => InterpreterType {
                name: "macro",
                is_macro: false,
            },
        }
    }

    pub fn check_type(&self, ty: &InterpreterType) -> bool {
        if ty.name == "any" {
            return true;
        }
        match self {
            Self::Int(_) => ty.name == "int" || ty.name == "number",
            Self::Float(_) => ty.name == "float" || ty.name == "number",
            Self::String(_) => ty.name == "string",
            Self::Bool(_) => ty.name == "bool",
            Self::Array(_) => ty.name == "array",
            Self::Type(_) => ty.name == "type",
            Self::Void => ty.name == "void",
            Self::Function { .. } => ty.name == "function",
            Self::NativeFunction { .. } => ty.name == "function",
            Self::Macro { .. } => ty.name == "macro",
            Self::NativeMacro { .. } => ty.name == "macro",
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
            Self::Type(ty) => format!("${}", ty.name),
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
