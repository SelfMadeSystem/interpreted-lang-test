use crate::{lexer::{InstructionToken, InstructionTokenType}, value::Value};

#[derive(Debug, Clone)]
pub enum IrValue {
    /// A value.
    Value(Value),
    /// A variable.
    Var(String),
}

#[derive(Debug, Clone)]
pub enum Ir {
    /// Calls a function.
    Call { name: String, args: Vec<IrValue> },
    /// Calls a function and assigns the result to a variable.
    CallAssign {
        var: String,
        name: String,
        args: Vec<IrValue>,
    },
    /// Assigns a value to a variable.
    Assign { var: String, value: IrValue },
    /// Jumps to a label.
    Jump { label: String },
    /// Jumps to a label if a condition is true.
    JumpIf { label: String, cond: IrValue },
    /// Returns a value.
    Return { value: IrValue },
    /// Defines a label.
    Label { name: String },
}

impl IrValue {
    pub fn from_lex(token: &InstructionToken) -> Result<IrValue, anyhow::Error> {
        Ok(match &token.ty {
            InstructionTokenType::Identifier(name) => IrValue::Var(name.clone()),
            InstructionTokenType::Number(n) => IrValue::Value(Value::Float(*n)),
            InstructionTokenType::Boolean(b) => IrValue::Value(Value::Bool(*b)),
            InstructionTokenType::String(s) => IrValue::Value(Value::String(s.clone())),
            InstructionTokenType::Array(a) => IrValue::Value(Value::from_lexed_array(a)?),
            InstructionTokenType::Void => IrValue::Value(Value::Void),
        })
    }
}
