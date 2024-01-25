use std::{collections::HashMap, rc::Rc};

use anyhow::{Error, Result};

use crate::{
    ast::{AstNode, AstNodeType},
    token::Keyword,
};

#[derive(Debug, Clone, thiserror::Error)]
pub enum InterpreterError {
    // TODO: Add line and column numbers to all errors
    #[error("Variable {0} not found at {1}:{2}")]
    VariableNotFound(String, usize, usize),
    #[error("Function {0} not found at {1}:{2}")]
    FunctionNotFound(String, usize, usize),
    #[error("Invalid const value {0:?} at {1}:{2}")]
    InvalidConstValue(AstNode, usize, usize),
    #[error("Multiple main functions found. First at {0}:{1}, second at {2}:{3}")]
    MultipleMainFunctions(usize, usize, usize, usize),
    #[error("No main function found")]
    NoMainFunction,
    #[error("Main in inner scope. Found at {0}:{1}")]
    MainInInnerScope(usize, usize),
    #[error("Invalid function call for {0}")]
    InvalidFunctionCall(String),
    #[error("Invalid types {0} for {1}")]
    InvalidType1Native(String, String),
    #[error("Invalid types {0} and {1} for {2}")]
    InvalidType2Native(String, String, String),
    #[error("Invalid type {0} at argument {1} for {2}. Expected type: {3}")]
    InvalidTypeArgNative(String, usize, String, String),
}

pub type NativeFn = fn(&mut InterpreterScope, &Vec<AstNode>) -> Result<Rc<InterpreterValue>>;

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
        body: Box<AstNode>,
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
                format!("Function {{ name: {}, params: {:?} }}", name, params)
            }
            Self::NativeFunction { name, .. } => format!("NativeFunction {{ name: {} }}", name),
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
            AstNodeType::Array(value) => todo!(),
            AstNodeType::Fn { name, params, body } => todo!(),
            AstNodeType::Keyword(Keyword::True) => Ok(Self::Bool(true)),
            AstNodeType::Keyword(Keyword::False) => Ok(Self::Bool(false)),
            _ => Err(
                InterpreterError::InvalidConstValue(value.clone(), value.line, value.col).into(),
            ),
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
            match &node.ty {
                AstNodeType::Const { name, value } => {
                    self.top_scope
                        .variables
                        .insert(name.clone(), Rc::new((*value.clone()).try_into()?));
                }
                AstNodeType::Fn { name, params, body } => {
                    self.top_scope.variables.insert(
                        name.clone(),
                        Rc::new(InterpreterValue::Function {
                            name: name.clone(),
                            params: params
                                .clone()
                                .into_iter()
                                .map(|p| match p {
                                    AstNode {
                                        ty: AstNodeType::Ident(i),
                                        ..
                                    } => i,
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

    fn find_main(&self) -> Result<AstNode> {
        let mut main: Option<(AstNode, AstNode)> = None;

        for node in self.ast.iter() {
            match &node.ty {
                AstNodeType::Main(nodes) => {
                    if main.is_some() {
                        return Err(InterpreterError::MultipleMainFunctions(
                            main.as_ref().unwrap().0.line,
                            main.as_ref().unwrap().0.col,
                            nodes.line,
                            nodes.col,
                        )
                        .into());
                    }
                    main = Some((node.clone(), *nodes.clone()));
                }
                _ => {}
            }
        }

        Ok(main.map(|m| m.1).ok_or(InterpreterError::NoMainFunction)?)
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
    if parent.is_null() {
        panic!("Parent is null");
    }
    unsafe { &mut **parent }
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

    pub fn get(&self, name: &str, line: usize, col: usize) -> Result<Rc<InterpreterValue>> {
        if let Some(value) = self.variables.get(name) {
            return Ok(value.clone());
        }

        if let Some(parent) = self.parent.as_ref() {
            return g(parent).get(name, line, col);
        }

        Err(InterpreterError::VariableNotFound(name.to_string(), line, col).into())
    }

    pub fn set(&mut self, name: &str, value: Rc<InterpreterValue>) -> Result<()> {
        self.variables.insert(name.to_string(), value);
        Ok(())
    }

    pub fn replace(
        &mut self,
        name: &str,
        value: Rc<InterpreterValue>,
        line: usize,
        col: usize,
    ) -> Result<()> {
        if self.variables.contains_key(name) {
            self.variables.insert(name.to_string(), value);
            return Ok(());
        }

        if let Some(parent) = self.parent.as_ref() {
            return g(parent).replace(name, value, line, col);
        }

        Err(InterpreterError::VariableNotFound(name.to_string(), line, col).into())
    }

    fn dbg_print_vars(&self) {
        println!("Variables: {:#?}", self.variables);
        if let Some(parent) = self.parent.as_ref() {
            g(parent).dbg_print_vars();
        }
    }

    pub fn evaluate(&mut self, node: &AstNode) -> Result<Rc<InterpreterValue>> {
        match &node.ty {
            AstNodeType::Int(value) => Ok(Rc::new(InterpreterValue::Int(*value))),
            AstNodeType::Float(value) => Ok(Rc::new(InterpreterValue::Float(*value))),
            AstNodeType::String(value) => Ok(Rc::new(InterpreterValue::String(value.clone()))),
            AstNodeType::Bool(b) => Ok(Rc::new(InterpreterValue::Bool(*b))),
            AstNodeType::Array(value) => todo!(),
            AstNodeType::Fn { name, params, body } => {
                let function = Rc::new(InterpreterValue::Function {
                    name: name.clone(),
                    params: params
                        .clone()
                        .into_iter()
                        .map(|p| match p.ty {
                            AstNodeType::Ident(i) => i,
                            _ => panic!(), // TODO: Better error handling
                        })
                        .collect(),
                    body: body.clone(),
                });
                if !name.contains(" ") {
                    // no spaces allowed in function names
                    self.set(&name, function.clone())?;
                }
                Ok(function)
            }
            AstNodeType::Const { name, value } => {
                // TODO: Allow for immutable variables
                let value = self.evaluate(&value)?;
                self.set(&name, value.clone())?;
                Ok(value)
            }
            AstNodeType::Let { name, value } => {
                let value = self.evaluate(&value)?;
                self.set(&name, value.clone())?;
                Ok(value)
            }
            AstNodeType::Set { name, value: node } => {
                let value = self.evaluate(&node)?;
                self.replace(&name, value.clone(), node.line, node.col)?;
                Ok(value)
            }
            AstNodeType::If {
                condition,
                body,
                else_body,
            } => {
                let condition = self.evaluate(&condition)?;
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
                    self.evaluate(&body)
                } else {
                    match else_body {
                        Some(else_body) => self.evaluate(&else_body),
                        None => Ok(Rc::new(InterpreterValue::Void)),
                    }
                }
            }
            AstNodeType::While { condition, body } => {
                let mut result = Rc::new(InterpreterValue::Void);
                loop {
                    let condition = self.evaluate(&condition)?;
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
                    result = self.evaluate(&body)?;
                }
            }
            AstNodeType::Main(_) => {
                Err(InterpreterError::MainInInnerScope(node.line, node.col).into())
            }
            AstNodeType::Call { name, params } => {
                let function = self.get(&name, node.line, node.col);
                let function = match function {
                    Ok(function) => function,
                    Err(_) => {
                        self.dbg_print_vars();
                        return Err(InterpreterError::FunctionNotFound(
                            name.to_string(),
                            node.line,
                            node.col,
                        )
                        .into());
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
                        Ok(scope.evaluate(&body)?)
                    }
                    InterpreterValue::NativeFunction { body, .. } => body(self, params),
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
            AstNodeType::Block(nodes) => {
                let mut scope = InterpreterScope::new_child(self);
                scope.evaluate_block(&nodes)
            }
            AstNodeType::Ident(ident) => {
                let value = self.get(&ident, node.line, node.col)?;
                Ok(value)
            }
            AstNodeType::Keyword(keyword) => todo!("{:?}", keyword),
        }
    }

    pub fn evaluate_block(&mut self, nodes: &[AstNode]) -> Result<Rc<InterpreterValue>> {
        let mut result = Rc::new(InterpreterValue::Void);
        for node in nodes.iter() {
            result = self.evaluate(node)?;
        }
        Ok(result)
    }

    pub fn evaluate_each(&mut self, nodes: &[AstNode]) -> Result<Vec<Rc<InterpreterValue>>> {
        let mut result = Vec::new();
        for node in nodes.iter() {
            result.push(self.evaluate(node)?);
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

    let result = interpreter.top_scope.evaluate(&main)?;

    Ok(result)
}
