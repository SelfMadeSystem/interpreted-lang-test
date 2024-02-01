use crate::{
    lexer::{InstructionToken, InstructionTokenType},
    value::Value,
};

#[derive(Debug, Clone)]
pub enum IrValue {
    /// A value.
    Value(Value),
    /// A variable.
    Var(usize),
}

#[derive(Debug, Clone)]
pub enum Ir {
    /// Calls a function.
    Call { name: String, args: Vec<IrValue> },
    /// Calls a function and assigns the result to a variable.
    CallAssign {
        var: usize,
        name: String,
        args: Vec<IrValue>,
    },
    /// Assigns a value to a variable.
    Assign { var: usize, value: IrValue },
    /// Jumps to a line.
    Jump { line: usize },
    /// Jumps to a line if a condition is true.
    JumpIf { line: usize, cond: IrValue },
    /// Returns a value.
    Return { value: IrValue },
}

impl IrValue {
    pub fn from_lex(token: &InstructionToken) -> Result<IrValue, anyhow::Error> {
        Ok(match &token.ty {
            InstructionTokenType::Identifier(_) => unreachable!(),
            InstructionTokenType::Int(n) => IrValue::Value(Value::Int(*n)),
            InstructionTokenType::Float(f) => IrValue::Value(Value::Float(*f)),
            InstructionTokenType::Boolean(b) => IrValue::Value(Value::Bool(*b)),
            InstructionTokenType::String(s) => IrValue::Value(Value::String(s.clone())),
            InstructionTokenType::Array(a) => IrValue::Value(Value::from_lexed_array(a)?),
            InstructionTokenType::Void => IrValue::Value(Value::Void),
        })
    }
}
