use std::collections::HashMap;

use anyhow::Result;
use thiserror::Error;

use crate::{
    ir::{Ir, IrValue},
    lexer::{InstructionToken, InstructionTokenType, InstructionType, Line, LineType},
    value::Value,
};

#[derive(Debug, Clone, Error)]
pub enum ParseError {
    #[error("Invalid line type {0:#?} at line {1}. Expected function")]
    InvalidLineTypeEF(LineType, usize),
    #[error("Invalid IR value `{0}` at line {1}, column {2}")]
    InvalidIRValue(String, usize, usize),
    #[error("Label `{0}` not found at line {1}")]
    LabelNotFound(String, usize),
}

#[derive(Debug, Clone)]
pub struct Function {
    pub name: String,
    pub args: usize,
    pub body: Vec<Ir>,
}

#[derive(Debug, Clone)]
pub struct Parser {
    lines: Vec<Line>,
    index: usize,
    vars: Vec<String>,
}

impl Parser {
    pub fn new(lines: &[Line]) -> Self {
        Self {
            lines: lines.to_vec(),
            index: 0,
            vars: Vec::new(),
        }
    }

    pub fn get_var(&mut self, name: &str) -> usize {
        if let Some(index) = self.vars.iter().position(|var| var == name) {
            index
        } else {
            self.vars.push(name.to_string());
            self.vars.len() - 1
        }
    }

    pub fn ir_var_from_lex(&mut self, token: &InstructionToken) -> Result<IrValue> {
        match &token.ty {
            InstructionTokenType::Identifier(name) => Ok(IrValue::Var(self.get_var(&name))),
            _ => Ok(IrValue::from_lex(token)?),
        }
    }

    pub fn parse(&mut self) -> Result<Vec<Function>> {
        let mut functions = Vec::new();
        while self.index < self.lines.len() {
            let line = &self.lines[self.index];
            match line.ty.clone() {
                LineType::Function { name, args } => {
                    for arg in args.iter() {
                        self.get_var(arg);
                    }

                    let labels = self.find_labels()?;
                    let body = self.parse_function(labels)?;
                    functions.push(Function {
                        name: name.clone(),
                        args: args.len(),
                        body,
                    });
                }
                LineType::Comment { .. } => {}
                a => return Err(ParseError::InvalidLineTypeEF(a, line.line).into()),
            }
            self.index += 1;
            self.vars.clear();
        }
        Ok(functions)
    }

    fn find_labels(&self) -> Result<HashMap<String, usize>> {
        let mut labels = HashMap::new();
        let mut sub = self.index;
        for (i, line) in self.lines.iter().enumerate().skip(self.index + 1) {
            match line.ty.clone() {
                LineType::Label { name } => {
                    sub += 1;
                    labels.insert(name, i - sub);
                }
                LineType::Function { .. } => break,
                _ => {}
            }
        }
        Ok(labels)
    }

    fn parse_function(&mut self, labels: HashMap<String, usize>) -> Result<Vec<Ir>> {
        let mut body = Vec::new();
        self.index += 1;
        while self.index < self.lines.len() {
            let line = &self.lines[self.index];
            self.index += 1;
            body.push(match line.ty.clone() {
                LineType::Label { .. } => continue,
                LineType::Instruction { ty, tokens } => {
                    self.parse_instruction(&ty, &tokens, &labels)?
                }
                LineType::Comment { .. } => continue,
                LineType::Function { .. } => {
                    self.index -= 1;
                    break;
                }
            });
        }
        self.index -= 1;
        Ok(body)
    }

    fn parse_instruction(
        &mut self,
        ty: &InstructionType,
        tokens: &Vec<InstructionToken>,
        labels: &HashMap<String, usize>,
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
                    .map(|token| self.ir_var_from_lex(token))
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
                let var = self.get_var(&var);
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
                    .map(|token| self.ir_var_from_lex(token))
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
                let var = self.get_var(&var);
                let value = tokens[1].clone();
                let value = self.ir_var_from_lex(&value)?;
                Ok(Ir::Assign { var, value })
            }
            InstructionType::Jump => {
                let label = tokens[0].clone();
                let label_name = match label.ty {
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
                let line = *labels
                    .get(&label_name)
                    .ok_or_else(|| ParseError::LabelNotFound(label_name.to_string(), label.line))?;
                Ok(Ir::Jump { line })
            }
            InstructionType::JumpIf => {
                let cond = tokens[0].clone();
                let cond = match cond.ty {
                    InstructionTokenType::Identifier(cond) => IrValue::Var(self.get_var(&cond)),
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
                let label_name = match label.ty {
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
                let line = *labels
                    .get(&label_name)
                    .ok_or_else(|| ParseError::LabelNotFound(label_name.to_string(), label.line))?;
                Ok(Ir::JumpIf { cond, line })
            }
            InstructionType::Return => {
                let value = tokens[0].clone();
                let value = self.ir_var_from_lex(&value)?;
                Ok(Ir::Return { value })
            }
        }
    }
}
