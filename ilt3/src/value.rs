use std::{cell::RefCell, collections::HashMap, rc::Rc};

use anyhow::Result;

use crate::{
    ir::Ir,
    lexer::{InstructionToken, InstructionTokenType},
};

type NativeVaueFnBody = fn(Vec<Rc<RefCell<Value>>>) -> Result<Rc<RefCell<Value>>>;

#[derive(Debug, Clone)]
pub enum ValueFunctionBody {
    Ir(Vec<Ir>),
    Native(NativeVaueFnBody),
}

impl ValueFunctionBody {
    pub fn as_ir(&self) -> Option<&Vec<Ir>> {
        if let ValueFunctionBody::Ir(value) = self {
            Some(value)
        } else {
            None
        }
    }

    pub fn as_native(&self) -> Option<NativeVaueFnBody> {
        if let ValueFunctionBody::Native(value) = self {
            Some(*value)
        } else {
            None
        }
    }
}

#[derive(Debug, Clone)]
pub struct ValueFunction {
    pub args: usize,
    pub body: ValueFunctionBody,
}

#[derive(Debug, Clone)]
pub enum Value {
    Int(i64),
    Float(f64),
    String(String),
    Bool(bool),
    Array(Vec<Rc<RefCell<Value>>>),
    Dict(HashMap<String, Rc<RefCell<Value>>>),
    Void,
    Function(ValueFunction),
}

impl Value {
    pub fn as_function(&self) -> Option<&ValueFunction> {
        if let Value::Function(value) = self {
            Some(value)
        } else {
            None
        }
    }

    pub fn as_int(&self) -> Option<i64> {
        if let Value::Int(value) = self {
            Some(*value)
        } else {
            None
        }
    }

    pub fn as_float(&self) -> Option<f64> {
        if let Value::Float(value) = self {
            Some(*value)
        } else {
            None
        }
    }

    pub fn as_string(&self) -> Option<&str> {
        if let Value::String(value) = self {
            Some(value)
        } else {
            None
        }
    }

    pub fn as_bool(&self) -> Option<bool> {
        if let Value::Bool(value) = self {
            Some(*value)
        } else {
            None
        }
    }

    pub fn as_array(&self) -> Option<&Vec<Rc<RefCell<Value>>>> {
        if let Value::Array(value) = self {
            Some(value)
        } else {
            None
        }
    }

    pub fn as_dict(&self) -> Option<&HashMap<String, Rc<RefCell<Value>>>> {
        if let Value::Dict(value) = self {
            Some(value)
        } else {
            None
        }
    }

    pub fn from_lexed_array(a: &[InstructionToken]) -> Result<Value> {
        let mut array = Vec::new();
        for token in a {
            array.push(Rc::new(RefCell::new(match &token.ty {
                InstructionTokenType::Int(n) => Value::Int(*n),
                InstructionTokenType::Float(f) => Value::Float(*f),
                InstructionTokenType::Boolean(b) => Value::Bool(*b),
                InstructionTokenType::String(s) => Value::String(s.clone()),
                InstructionTokenType::Array(a) => Value::from_lexed_array(a)?,
                InstructionTokenType::Void => Value::Void,
                _ => return Err(anyhow::anyhow!("Invalid value in array")),
            })));
        }
        Ok(Value::Array(array))
    }
}
