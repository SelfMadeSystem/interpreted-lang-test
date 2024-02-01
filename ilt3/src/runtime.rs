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
    pub ir: Vec<Ir>,
}

impl Runtime {
    pub fn from_ast(ast: Vec<Function>) -> Self {
        let mut scope = Scope::new();
        for function in ast {
            let value = Value::Function(ValueFunction {
                args: function.args,
                body: ValueFunctionBody::Ir(function.body),
            });
            scope.set_named(function.name, Rc::new(RefCell::new(value)));
        }

        let ir = scope
            .get_named("main")
            .expect("No main function")
            .borrow()
            .as_function()
            .expect("main is not a function")
            .body
            .as_ir()
            .expect("main is not an IR function")
            .clone();
        Self {
            fn_name: "main".to_string(),
            scope: Rc::new(RefCell::new(scope)),
            ir,
        }
    }

    pub fn new(name: &String, scope: Rc<RefCell<Scope>>, ir: Vec<Ir>) -> Self {
        Self {
            fn_name: name.to_owned(),
            scope,
            ir,
        }
    }

    pub fn add_builtin_functions(&mut self) {
        add_builtin_functions(&mut self.scope.borrow_mut());
    }

    pub fn get_named(&self, name: &str) -> Result<Rc<RefCell<Value>>> {
        if let Some(value) = self.scope.borrow().get_named(name) {
            return Ok(value);
        }

        Err(RuntimeException::UndefinedVariable(name.to_string()).into())
    }

    pub fn get_local(&self, index: usize) -> Result<Rc<RefCell<Value>>> {
        if let Some(value) = self.scope.borrow().get_local(index) {
            return Ok(value);
        }

        Err(RuntimeException::UndefinedVariable(index.to_string()).into())
    }

    pub fn get_ir_value(&self, value: &IrValue) -> Result<Rc<RefCell<Value>>> {
        match value {
            IrValue::Value(value) => Ok(Rc::new(RefCell::new(value.clone()))),
            IrValue::Var(index) => self.get_local(*index),
        }
    }

    pub fn set_named(&mut self, name: String, value: Rc<RefCell<Value>>) {
        self.scope.borrow_mut().set_named(name, value);
    }

    pub fn set_local(&mut self, index: usize, value: Rc<RefCell<Value>>) {
        self.scope.borrow_mut().set_local(index, value);
    }

    pub fn run(&mut self) -> Result<Rc<RefCell<Value>>> {
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

                    let function = self.get_named(name)?;
                    let function = function.borrow();
                    let function = function
                        .as_function()
                        .ok_or(RuntimeException::WrongType("function".to_string()))?;

                    match function.body {
                        ValueFunctionBody::Ir(ref ir) => {
                            let mut scope = Scope::new_child(self.scope.clone());
                            for i in 0..function.args {
                                scope.set_local(i, values[i].clone());
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

                    let function = self.get_named(name)?;
                    let function = function.borrow();
                    let function = function
                        .as_function()
                        .ok_or(RuntimeException::WrongType("function".to_string()))?;

                    match function.body {
                        ValueFunctionBody::Ir(ref ir) => {
                            let mut scope = Scope::new_child(self.scope.clone());
                            for i in 0..function.args {
                                scope.set_local(i, values[i].clone());
                            }

                            let mut runtime =
                                Runtime::new(name, Rc::new(RefCell::new(scope)), ir.clone());
                            let value = runtime.run()?;
                            self.set_local(*var, value);
                        }
                        ValueFunctionBody::Native(ref native) => {
                            let value = native(values)?;
                            self.set_local(*var, value);
                        }
                    }
                }
                Ir::Assign { var, value } => {
                    let value = self.get_ir_value(value)?;
                    self.set_local(*var, value);
                }
                Ir::Jump { line } => {
                    ip = *line;
                    continue;
                }
                Ir::JumpIf { line, cond } => {
                    let cond = self.get_ir_value(cond)?;
                    if cond
                        .borrow()
                        .as_bool()
                        .ok_or(RuntimeException::WrongType("bool".to_string()))?
                    {
                        ip = *line;
                        continue;
                    }
                }
                Ir::Return { value } => {
                    return Ok(self.get_ir_value(value)?);
                }
            }

            ip += 1;
        }

        Err(RuntimeException::NoReturnValue(self.fn_name.clone()).into())
    }
}
