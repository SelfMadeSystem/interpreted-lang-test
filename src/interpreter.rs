use std::{collections::HashMap, rc::Rc};

use anyhow::{Error, Result};

use crate::{ast::AstNode, token::Keyword};

#[derive(Debug, Clone, thiserror::Error)]
pub enum InterpreterError {
    #[error("Variable {0} not found")]
    VariableNotFound(String),
    #[error("Function {0} not found")]
    FunctionNotFound(String),
    #[error("Invalid const value {0:?}")]
    InvalidConstValue(AstNode),
    #[error("Multiple main functions found")]
    MultipleMainFunctions,
    #[error("No main function found")]
    NoMainFunction,
    #[error("Main in inner scope")]
    MainInInnerScope,
    #[error("Invalid function call for {0}")]
    InvalidFunctionCall(String),
    #[error("Invalid types {0} for {1}")]
    InvalidType1Native(String, String),
    #[error("Invalid types {0} {1} for {2}")]
    InvalidType2Native(String, String, String),
}

pub type NativeFn =
    fn(&mut InterpreterScope, Vec<Rc<InterpreterValue>>) -> Result<Rc<InterpreterValue>>;

#[derive(Debug, Clone)]
pub enum InterpreterValue {
    Int(i64),
    Float(f64),
    String(String),
    Bool(bool),
    Array(Vec<InterpreterValue>),
    Void,
    Function {
        name: String,
        params: Vec<String>,
        body: Vec<AstNode>,
    },
    NativeFunction {
        name: String,
        body: NativeFn,
    },
}

impl InterpreterValue {
    pub fn get_type(&self) -> &'static str {
        match self {
            Self::Int(_) => "int",
            Self::Float(_) => "float",
            Self::String(_) => "string",
            Self::Bool(_) => "bool",
            Self::Array(_) => "array",
            Self::Void => "void",
            Self::Function { .. } => "function",
            Self::NativeFunction { .. } => "native_function",
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
                    .map(|v| v.to_string())
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            Self::Void => "Void".to_string(),
            Self::Function { name, params, .. } => {
                format!("Function {{ name: {}, params: {:?}, }}", name, params)
            }
            Self::NativeFunction { name, .. } => format!("NativeFunction {{ name: {} }}", name),
        }
    }
}

impl TryFrom<AstNode> for InterpreterValue {
    type Error = Error;

    fn try_from(value: AstNode) -> Result<Self, Self::Error> {
        match value {
            AstNode::Int(value) => Ok(Self::Int(value)),
            AstNode::Float(value) => Ok(Self::Float(value)),
            AstNode::String(value) => Ok(Self::String(value)),
            AstNode::Array(value) => todo!(),
            AstNode::Fn { name, params, body } => todo!(),
            AstNode::Keyword(Keyword::True) => Ok(Self::Bool(true)),
            AstNode::Keyword(Keyword::False) => Ok(Self::Bool(false)),
            _ => Err(InterpreterError::InvalidConstValue(value).into()),
        }
    }
}

#[derive(Debug)]
pub struct Interpreter {
    pub(crate) ast: Vec<AstNode>,
    pub(crate) top_scope: InterpreterScope,
}

impl Interpreter {
    fn find_constants(&mut self) -> Result<()> {
        for node in self.ast.iter() {
            match node {
                AstNode::Const { name, value } => {
                    self.top_scope
                        .variables
                        .insert(name.clone(), Rc::new((*value.clone()).try_into()?));
                }
                AstNode::Fn { name, params, body } => {
                    self.top_scope.variables.insert(
                        name.clone(),
                        Rc::new(InterpreterValue::Function {
                            name: name.clone(),
                            params: params
                                .clone()
                                .into_iter()
                                .map(|p| match p {
                                    AstNode::Ident(i) => i,
                                    _ => unreachable!(),
                                })
                                .collect(),
                            body: body.clone(),
                        }),
                    );
                }
                _ => {}
            }
        }

        Ok(())
    }

    fn find_main(&self) -> Result<Vec<AstNode>> {
        let mut main = None;

        for node in self.ast.iter() {
            match node {
                AstNode::Main(nodes) => {
                    if main.is_some() {
                        return Err(InterpreterError::MultipleMainFunctions.into());
                    }
                    main = Some(nodes.clone());
                }
                _ => {}
            }
        }

        Ok(main.ok_or(InterpreterError::NoMainFunction)?)
    }
}

#[derive(Debug)]
pub struct InterpreterScope {
    pub(crate) parent: Option<*mut InterpreterScope>,
    pub(crate) variables: HashMap<String, Rc<InterpreterValue>>,
}

/// I know this is unsafe, but I'm not sure how to do it otherwise without
/// making the code more complicated.
fn g<'a>(parent: &*mut InterpreterScope) -> &'a mut InterpreterScope {
    unsafe { parent.as_mut().unwrap() }
}

impl InterpreterScope {
    pub fn new() -> Self {
        Self {
            parent: None,
            variables: HashMap::new(),
        }
    }

    pub fn new_child(&self) -> Self {
        Self {
            parent: Some(self as *const InterpreterScope as *mut InterpreterScope),
            variables: HashMap::new(),
        }
    }

    pub fn get(&self, name: &str) -> Result<Rc<InterpreterValue>> {
        if let Some(value) = self.variables.get(name) {
            return Ok(value.clone());
        }

        if let Some(parent) = self.parent.as_ref() {
            return g(parent).get(name);
        }

        Err(InterpreterError::VariableNotFound(name.to_string()).into())
    }

    pub fn set(&mut self, name: &str, value: Rc<InterpreterValue>) -> Result<()> {
        self.variables.insert(name.to_string(), value);
        Ok(())
    }

    pub fn replace(&mut self, name: &str, value: Rc<InterpreterValue>) -> Result<()> {
        if self.variables.contains_key(name) {
            self.variables.insert(name.to_string(), value);
            return Ok(());
        }

        if let Some(parent) = self.parent.as_ref() {
            return g(parent).replace(name, value);
        }

        Err(InterpreterError::VariableNotFound(name.to_string()).into())
    }

    fn dbg_print_vars(&self) {
        println!("Variables: {:#?}", self.variables);
        if let Some(parent) = self.parent.as_ref() {
            g(parent).dbg_print_vars();
        }
    }

    pub fn evaluate(&mut self, node: &AstNode) -> Result<Rc<InterpreterValue>> {
        match node {
            AstNode::Int(value) => Ok(Rc::new(InterpreterValue::Int(*value))),
            AstNode::Float(value) => Ok(Rc::new(InterpreterValue::Float(*value))),
            AstNode::String(value) => Ok(Rc::new(InterpreterValue::String(value.clone()))),
            AstNode::Bool(b) => Ok(Rc::new(InterpreterValue::Bool(*b))),
            AstNode::Array(value) => todo!(),
            AstNode::Fn { name, params, body } => {
                let function = Rc::new(InterpreterValue::Function {
                    name: name.clone(),
                    params: params
                        .clone()
                        .into_iter()
                        .map(|p| match p {
                            AstNode::Ident(i) => i,
                            _ => unreachable!(),
                        })
                        .collect(),
                    body: body.clone(),
                });
                if !name.contains(" ") { // no spaces allowed in function names
                    self.set(&name, function.clone())?;
                }
                Ok(function)
            }
            AstNode::Const { name, value } => {
                // TODO: Allow for immutable variables
                let value = self.evaluate(value)?;
                self.set(&name, value.clone())?;
                Ok(value)
            }
            AstNode::Let { name, value } => {
                let value = self.evaluate(value)?;
                self.set(&name, value.clone())?;
                Ok(value)
            }
            AstNode::Set { name, value } => {
                let value = self.evaluate(value)?;
                self.replace(&name, value.clone())?;
                Ok(value)
            }
            AstNode::If {
                condition,
                body,
                else_body,
            } => {
                let condition = self.evaluate(condition)?;
                let condition = match condition.as_ref() {
                    InterpreterValue::Bool(b) => *b,
                    _ => {
                        return Err(InterpreterError::InvalidType1Native(
                            condition.get_type().to_string(),
                            "if".to_string(),
                        )
                        .into());
                    }
                };
                if condition {
                    self.evaluate(body)
                } else {
                    match else_body {
                        Some(else_body) => self.evaluate(else_body),
                        None => Ok(Rc::new(InterpreterValue::Void)),
                    }
                }
            }
            AstNode::While { condition, body } => {
                let mut result = Rc::new(InterpreterValue::Void);
                loop {
                    let condition = self.evaluate(condition)?;
                    let condition = match condition.as_ref() {
                        InterpreterValue::Bool(b) => *b,
                        _ => {
                            return Err(InterpreterError::InvalidType1Native(
                                condition.get_type().to_string(),
                                "while".to_string(),
                            )
                            .into());
                        }
                    };
                    if !condition {
                        break Ok(result);
                    }
                    result = self.evaluate(body)?;
                }
            }
            AstNode::Main(_) => Err(InterpreterError::MainInInnerScope.into()),
            AstNode::Call { name, params } => {
                let function = self.get(&name);
                let function = match function {
                    Ok(function) => function,
                    Err(_) => {
                        self.dbg_print_vars();
                        return Err(InterpreterError::FunctionNotFound(name.to_string()).into());
                    }
                };
                match function.as_ref() {
                    InterpreterValue::Function {
                        name,
                        params: fn_params,
                        body,
                    } => {
                        if params.len() != fn_params.len() {
                            return Err(
                                InterpreterError::InvalidFunctionCall(name.to_owned()).into()
                            );
                        }
                        let mut scope = InterpreterScope::new_child(self);
                        for (param, value) in fn_params.iter().zip(params.iter()) {
                            let value = scope.evaluate(value)?;
                            scope.set(param, value)?;
                        }
                        Ok(scope.evaluate_block(body)?)
                    }
                    InterpreterValue::NativeFunction { body, .. } => {
                        let mut values = Vec::new();
                        for param in params.iter() {
                            let value = self.evaluate(param)?;
                            values.push(value);
                        }
                        body(self, values)
                    }
                    _ => {
                        if params.len() != 0 {
                            return Err(
                                InterpreterError::InvalidFunctionCall(name.to_owned()).into()
                            );
                        }
                        return Ok(function);
                    }
                }
            }
            AstNode::Block(nodes) => self.evaluate_block(nodes),
            AstNode::Ident(ident) => {
                let value = self.get(ident)?;
                Ok(value)
            }
            AstNode::Keyword(keyword) => todo!("{:?}", keyword),
        }
    }

    pub fn evaluate_block(&mut self, nodes: &[AstNode]) -> Result<Rc<InterpreterValue>> {
        let mut result = Rc::new(InterpreterValue::Void);
        for node in nodes.iter() {
            result = self.evaluate(node)?;
        }
        Ok(result)
    }
}

pub fn interpret(
    ast: Vec<AstNode>,
    functions: HashMap<String, NativeFn>,
) -> Result<Rc<InterpreterValue>> {
    let mut interpreter = Interpreter {
        ast,
        top_scope: InterpreterScope::new(),
    };

    for (name, function) in functions {
        interpreter.top_scope.set(
            &name,
            Rc::new(InterpreterValue::NativeFunction {
                name: name.clone(),
                body: function,
            }),
        )?;
    }

    interpreter.find_constants()?;

    let main = interpreter.find_main()?;

    let result = interpreter.top_scope.evaluate_block(&main)?;

    Ok(result)
}
