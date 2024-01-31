use anyhow::Result;
use thiserror::Error;

use crate::{
    ir::{Ir, IrValue},
    lexer::{InstructionToken, InstructionTokenType, InstructionType, Line, LineType},
    value::Value,
};

#[derive(Debug, Clone, Error)]
pub enum ParseError {
    #[error("Invalid line type at line {0}. Expected function")]
    InvalidLineTypeEF(usize),
    #[error("Invalid IR value `{0}` at line {1}, column {2}")]
    InvalidIRValue(String, usize, usize),
}

#[derive(Debug, Clone)]
pub struct Function {
    pub name: String,
    pub args: Vec<String>,
    pub body: Vec<Ir>,
}

#[derive(Debug, Clone)]
pub struct Parser {
    lines: Vec<Line>,
    index: usize,
}

impl Parser {
    pub fn new(lines: &[Line]) -> Self {
        Self {
            lines: lines.to_vec(),
            index: 0,
        }
    }

    pub fn parse(&mut self) -> Result<Vec<Function>> {
        let mut functions = Vec::new();
        while self.index < self.lines.len() {
            let line = &self.lines[self.index];
            match line.ty.clone() {
                LineType::Function { name, args } => {
                    let body = self.parse_function()?;
                    functions.push(Function {
                        name: name.clone(),
                        args: args.clone(),
                        body,
                    });
                }
                LineType::Comment { .. } => {}
                _ => return Err(ParseError::InvalidLineTypeEF(self.index).into()),
            }
            self.index += 1;
        }
        Ok(functions)
    }

    fn parse_function(&mut self) -> Result<Vec<Ir>> {
        let mut body = Vec::new();
        self.index += 1;
        while self.index < self.lines.len() {
            let line = &self.lines[self.index];
            body.push(match line.ty.clone() {
                LineType::Label { name } => Ir::Label { name: name.clone() },
                LineType::Instruction { ty, tokens } => self.parse_instruction(&ty, &tokens)?,
                LineType::Comment { .. } => continue,
                LineType::Function { .. } => break,
            });
            self.index += 1;
        }
        self.index -= 1;
        Ok(body)
    }

    fn parse_instruction(
        &self,
        ty: &InstructionType,
        tokens: &Vec<InstructionToken>,
    ) -> Result<Ir> {
        match ty {
            InstructionType::Call => {
                let name = tokens[0].clone();
                let name = match name.ty {
                    InstructionTokenType::Identifier(name) => name,
                    _ => {
                        return Err(ParseError::InvalidIRValue(
                            name.to_string(),
                            name.line,
                            name.col,
                        )
                        .into())
                    }
                };
                let args = tokens[1..]
                    .iter()
                    .map(|token| IrValue::from_lex(token))
                    .collect::<Result<_>>()?;
                Ok(Ir::Call { name, args })
            }
            InstructionType::CallAssign => {
                let var = tokens[0].clone();
                let var = match var.ty {
                    InstructionTokenType::Identifier(var) => var,
                    _ => {
                        return Err(
                            ParseError::InvalidIRValue(var.to_string(), var.line, var.col).into(),
                        )
                    }
                };
                let name = tokens[1].clone();
                let name = match name.ty {
                    InstructionTokenType::Identifier(name) => name,
                    _ => {
                        return Err(ParseError::InvalidIRValue(
                            name.to_string(),
                            name.line,
                            name.col,
                        )
                        .into())
                    }
                };
                let args = tokens[2..]
                    .iter()
                    .map(|token| IrValue::from_lex(token))
                    .collect::<Result<_>>()?;
                Ok(Ir::CallAssign { var, name, args })
            }
            InstructionType::Assign => {
                let var = tokens[0].clone();
                let var = match var.ty {
                    InstructionTokenType::Identifier(var) => var,
                    _ => {
                        return Err(
                            ParseError::InvalidIRValue(var.to_string(), var.line, var.col).into(),
                        )
                    }
                };
                let value = tokens[1].clone();
                let value = match value.ty {
                    InstructionTokenType::Identifier(value) => IrValue::Var(value),
                    _ => IrValue::from_lex(&value)?,
                };
                Ok(Ir::Assign { var, value })
            }
            InstructionType::Jump => {
                let label = tokens[0].clone();
                let label = match label.ty {
                    InstructionTokenType::Identifier(label) => label,
                    _ => {
                        return Err(ParseError::InvalidIRValue(
                            label.to_string(),
                            label.line,
                            label.col,
                        )
                        .into())
                    }
                };
                Ok(Ir::Jump { label })
            }
            InstructionType::JumpIf => {
                let cond = tokens[0].clone();
                let cond = match cond.ty {
                    InstructionTokenType::Identifier(cond) => IrValue::Var(cond),
                    InstructionTokenType::Boolean(b) => IrValue::Value(Value::Bool(b)),
                    _ => {
                        return Err(ParseError::InvalidIRValue(
                            cond.to_string(),
                            cond.line,
                            cond.col,
                        )
                        .into())
                    }
                };
                let label = tokens[1].clone();
                let label = match label.ty {
                    InstructionTokenType::Identifier(label) => label,
                    _ => {
                        return Err(ParseError::InvalidIRValue(
                            label.to_string(),
                            label.line,
                            label.col,
                        )
                        .into())
                    }
                };
                Ok(Ir::JumpIf { cond, label })
            }
            InstructionType::Return => {
                let value = tokens[0].clone();
                let value = match value.ty {
                    InstructionTokenType::Identifier(value) => IrValue::Var(value),
                    _ => IrValue::from_lex(&value)?,
                };
                Ok(Ir::Return { value })
            }
        }
    }
}
