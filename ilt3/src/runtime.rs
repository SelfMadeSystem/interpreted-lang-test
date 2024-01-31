use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use anyhow::Result;
use thiserror::Error;

use crate::builtin_functions::add_builtin_functions;
use crate::ir::{Ir, IrValue};
use crate::parser::Function;
use crate::scope::Scope;
use crate::value::{Value, ValueFunction, ValueFunctionBody};

#[derive(Debug, Clone, Error)]
pub enum RuntimeException {
    #[error("Undefined label: {0}")]
    UndefinedLabel(String),
    #[error("Undefined variable: {0}")]
    UndefinedVariable(String),
    #[error("Wrong type. Expected {0}")]
    WrongType(String),
    #[error("No return value in function {0}")]
    NoReturnValue(String),
}

#[derive(Debug, Clone)]
pub struct Runtime {
    pub fn_name: String,
    pub scope: Rc<RefCell<Scope>>,
    pub labels: HashMap<String, usize>,
    pub ir: Vec<Ir>,
}

fn find_labels(ir: &Vec<Ir>) -> HashMap<String, usize> {
    let mut labels = HashMap::new();

    for (i, ir) in ir.iter().enumerate() {
        if let Ir::Label { name } = ir {
            labels.insert(name.clone(), i);
        }
    }

    labels
}

impl Runtime {
    pub fn from_ast(ast: Vec<Function>) -> Self {
        let mut scope = Scope::new();
        for function in ast {
            let value = Value::Function(ValueFunction {
                args: function.args,
                body: ValueFunctionBody::Ir(function.body),
            });
            scope.set(function.name, value);
        }

        let ir = scope
            .get("main")
            .unwrap()
            .as_function()
            .unwrap()
            .body
            .as_ir()
            .unwrap()
            .clone();
        Self {
            fn_name: "main".to_string(),
            scope: Rc::new(RefCell::new(scope)),
            labels: HashMap::new(),
            ir,
        }
    }

    pub fn new(name: &String, scope: Rc<RefCell<Scope>>, ir: Vec<Ir>) -> Self {
        Self {
            fn_name: name.to_owned(),
            scope,
            labels: find_labels(&ir),
            ir,
        }
    }

    pub fn add_builtin_functions(&mut self) {
        add_builtin_functions(&mut self.scope.borrow_mut());
    }

    pub fn get(&self, name: &str) -> Result<Value> {
        if let Some(value) = self.scope.borrow().get(name) {
            return Ok(value);
        }

        Err(RuntimeException::UndefinedVariable(name.to_string()).into())
    }

    pub fn get_ir_value(&self, value: &IrValue) -> Result<Value> {
        match value {
            IrValue::Value(value) => Ok(value.clone()),
            IrValue::Var(name) => self.get(name),
        }
    }

    pub fn set(&mut self, name: String, value: Value) {
        self.scope.borrow_mut().set(name, value);
    }

    pub fn run(&mut self) -> Result<Value> {
        let mut ip = 0; // Instruction pointer

        loop {
            if ip >= self.ir.len() {
                break;
            }

            match &self.ir[ip].clone() {
                Ir::Call { name, args } => {
                    let mut values = vec![];

                    for arg in args {
                        values.push(self.get_ir_value(arg)?);
                    }

                    let function = self.get(name)?;
                    let function = function
                        .as_function()
                        .ok_or(RuntimeException::WrongType("function".to_string()))?;

                    match function.body {
                        ValueFunctionBody::Ir(ref ir) => {
                            let mut scope = Scope::new_child(self.scope.clone());
                            for (i, arg) in function.args.iter().enumerate() {
                                scope.set(arg.clone(), values[i].clone());
                            }

                            let mut runtime =
                                Runtime::new(name, Rc::new(RefCell::new(scope)), ir.clone());
                            runtime.run()?;
                        }
                        ValueFunctionBody::Native(ref native) => {
                            native(values)?;
                        }
                    }
                }
                Ir::CallAssign { var, name, args } => {
                    let mut values = vec![];

                    for arg in args {
                        values.push(self.get_ir_value(arg)?);
                    }

                    let function = self.get(name)?;
                    let function = function
                        .as_function()
                        .ok_or(RuntimeException::WrongType("function".to_string()))?;

                    match function.body {
                        ValueFunctionBody::Ir(ref ir) => {
                            let mut scope = Scope::new_child(self.scope.clone());
                            for (i, arg) in function.args.iter().enumerate() {
                                scope.set(arg.clone(), values[i].clone());
                            }

                            let mut runtime =
                                Runtime::new(name, Rc::new(RefCell::new(scope)), ir.clone());
                            let result = runtime.run()?;
                            self.set(var.clone(), result);
                        }
                        ValueFunctionBody::Native(ref native) => {
                            let result = native(values)?;
                            self.set(var.clone(), result);
                        }
                    }
                }
                Ir::Assign { var, value } => {
                    let value = self.get_ir_value(value)?;
                    self.set(var.clone(), value);
                }
                Ir::Jump { label } => {
                    ip = *self
                        .labels
                        .get(label)
                        .ok_or(RuntimeException::UndefinedLabel(label.clone()))?;
                    continue;
                }
                Ir::JumpIf { label, cond } => {
                    let cond = self.get_ir_value(cond)?;
                    if cond
                        .as_bool()
                        .ok_or(RuntimeException::WrongType("bool".to_string()))?
                    {
                        ip = *self
                            .labels
                            .get(label)
                            .ok_or(RuntimeException::UndefinedLabel(label.clone()))?;
                        continue;
                    }
                }
                Ir::Return { value } => {
                    return Ok(self.get_ir_value(value)?);
                }
                Ir::Label { .. } => {}
            }

            ip += 1;
        }

        Err(RuntimeException::NoReturnValue(self.fn_name.clone()).into())
    }
}
