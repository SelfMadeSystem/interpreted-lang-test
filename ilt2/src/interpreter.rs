use std::{collections::HashMap, rc::Rc};

use anyhow::{Error, Result};

use crate::{
    ast::{AstNode, AstNodeType},
    token::TokenIdent,
};

#[derive(Debug, Clone, thiserror::Error)]
pub enum InterpreterError {
    #[error("Variable {0} not found at {1}:{2}")]
    VariableNotFound(String, usize, usize),
    #[error("Function {0} not found at {1}:{2}")]
    FunctionNotFound(String, usize, usize),
    #[error("Invalid const value {0:?} at {1}:{2}")]
    InvalidConstValue(AstNode, usize, usize),
    #[error("Cannot set const value {0} at {1}:{2}")]
    CannotSetConstValue(String, usize, usize),
    #[error("Multiple main functions found. First at {0}:{1}, second at {2}:{3}")]
    MultipleMainFunctions(usize, usize, usize, usize),
    #[error("No main function found")]
    NoMainFunction,
    #[error("Main in inner scope. Found at {0}:{1}")]
    MainInInnerScope(usize, usize),
    #[error("Invalid function call for {0}")]
    InvalidFunctionCall(String),
    #[error("Invalid macro call for {0}")]
    InvalidMacroCall(String),
    #[error("Invalid types {0} for {1}")]
    InvalidType1Native(String, String),
    #[error("Invalid types {0} and {1} for {2}")]
    InvalidType2Native(String, String, String),
    #[error("Invalid type {0} at argument {1} for {2}. Expected type: {3}")]
    InvalidTypeArgNative(String, usize, String, String),
}

pub type NativeFn = fn(
    &mut InterpreterScope,
    Vec<Rc<InterpreterValue>>,
    line: usize,
    col: usize,
) -> Result<Rc<InterpreterValue>>;
pub type NativeMacro = fn(
    &mut InterpreterScope,
    &Vec<AstNode>,
    line: usize,
    col: usize,
) -> Result<Rc<InterpreterValue>>;

// TODO: Add types
#[derive(Debug, Clone)]
pub enum InterpreterValue {
    Int(i64),
    Float(f64),
    String(String),
    Bool(bool),
    Array(Vec<Rc<InterpreterValue>>),
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
            Self::Macro { .. } => "macro",
            Self::NativeMacro { .. } => "native_macro",
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

#[derive(Debug)]
pub struct Interpreter {
    pub(crate) ast: Vec<AstNode>,
    pub(crate) top_scope: InterpreterScope,
    pub(crate) main: Option<AstNode>,
}

impl Interpreter {
    pub fn run_top_level(&mut self) -> Result<()> {
        for node in self.ast.iter() {
            let AstNodeType::Call { name, params: _ } = &node.ty else {
                continue;
            };
            if let TokenIdent::Macro(s) = name {
                if s == "main" {
                    if let Some(main) = &self.main {
                        return Err(InterpreterError::MultipleMainFunctions(
                            main.line, main.col, node.line, node.col,
                        )
                        .into());
                    }
                    self.main = Some(node.clone());
                    continue;
                }
            }
            self.top_scope.evaluate(node)?;
        }
        Ok(())
    }

    pub fn find_main(&mut self) -> Result<Vec<AstNode>> {
        if let Some(main) = self.main.clone() {
            match main.ty {
                AstNodeType::Call { name, params } => {
                    if let TokenIdent::Macro(s) = name {
                        if s == "main" {
                            return Ok(params);
                        }
                    }
                }
                _ => {}
            }
        }
        Err(InterpreterError::NoMainFunction.into())
    }
}

#[derive(Debug)]
pub struct InterpreterScope {
    pub top_scope: bool,
    pub parent: Option<*mut InterpreterScope>,
    pub variables: HashMap<String, Rc<InterpreterValue>>,
    pub constants: HashMap<String, Rc<InterpreterValue>>,
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
            top_scope: true,
            parent: None,
            variables: HashMap::new(),
            constants: HashMap::new(),
        }
    }

    pub fn new_child(&self) -> Self {
        Self {
            top_scope: false,
            parent: Some(self as *const InterpreterScope as *mut InterpreterScope),
            variables: HashMap::new(),
            constants: HashMap::new(),
        }
    }

    pub fn get(&self, name: &str, line: usize, col: usize) -> Result<Rc<InterpreterValue>> {
        if let Some(value) = self.constants.get(name) {
            return Ok(value.clone());
        }

        if let Some(value) = self.variables.get(name) {
            return Ok(value.clone());
        }

        if let Some(parent) = self.parent.as_ref() {
            return g(parent).get(name, line, col);
        }

        Err(InterpreterError::VariableNotFound(name.to_string(), line, col).into())
    }

    pub fn set(
        &mut self,
        name: &str,
        value: Rc<InterpreterValue>,
        line: usize,
        col: usize,
    ) -> Result<()> {
        if self.constants.contains_key(name) {
            return Err(InterpreterError::CannotSetConstValue(name.to_string(), line, col).into());
        }

        self.variables.insert(name.to_string(), value);
        Ok(())
    }

    pub fn set_const(
        &mut self,
        name: &str,
        value: Rc<InterpreterValue>,
        line: usize,
        col: usize,
    ) -> Result<()> {
        if self.constants.contains_key(name) {
            return Err(InterpreterError::CannotSetConstValue(name.to_string(), line, col).into());
        }

        self.constants.insert(name.to_string(), value);
        Ok(())
    }

    fn dbg_print_vars(&self) {
        println!("Variables: {:#?}", self.variables);
        println!("Constants: {:#?}", self.constants);
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
            AstNodeType::Array(value) => {
                let mut array = Vec::new();
                for value in value.iter() {
                    array.push(self.evaluate(value)?);
                }
                Ok(Rc::new(InterpreterValue::Array(array)))
            }
            AstNodeType::Call { name, params } => {
                let function = self.get(name.as_str(), node.line, node.col);
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
                        let mut scope = self.new_child();
                        for (param, value) in fn_params.iter().zip(params.iter()) {
                            let value = scope.evaluate(value)?;
                            scope.set(param, value, node.line, node.col)?;
                        }
                        Ok(scope.evaluate_block(&body)?)
                    }
                    InterpreterValue::NativeFunction { body, .. } => {
                        let params = self.evaluate_each(params)?;
                        body(self, params, node.line, node.col)
                    }
                    InterpreterValue::Macro {
                        name,
                        params: fn_params,
                        body,
                    } => todo!(),
                    InterpreterValue::NativeMacro { body, .. } => {
                        body(self, params, node.line, node.col)
                    }
                    _ => {
                        if params.len() != 0 {
                            return Err(
                                InterpreterError::InvalidFunctionCall(name.to_string()).into()
                            );
                        }
                        return Ok(function);
                    }
                }
            }
            AstNodeType::Ident(ident) => {
                let value = self.get(ident.as_str(), node.line, node.col)?;
                Ok(value)
            }
        }
    }

    pub fn call_function(
        &mut self,
        name: &str,
        params: Vec<Rc<InterpreterValue>>,
        line: usize,
        col: usize,
    ) -> Result<Rc<InterpreterValue>> {
        let function = self.get(name, line, col)?;
        match function.as_ref() {
            InterpreterValue::Function {
                name,
                params: fn_params,
                body,
            } => {
                if params.len() != fn_params.len() {
                    return Err(InterpreterError::InvalidFunctionCall(name.to_owned()).into());
                }
                let mut scope = self.new_child();
                for (param, value) in fn_params.iter().zip(params.iter()) {
                    scope.set(param, value.clone(), line, col)?;
                }
                scope.evaluate_block(&body)
            }
            InterpreterValue::NativeFunction { body, .. } => {
                body(self, params, line, col)
            }
            _ => return Err(InterpreterError::InvalidFunctionCall(name.to_string()).into()),
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
    macros: HashMap<String, NativeMacro>,
) -> Result<Rc<InterpreterValue>> {
    let mut interpreter = Interpreter {
        ast,
        top_scope: InterpreterScope::new(),
        main: None,
    };

    for (name, function) in functions {
        interpreter.top_scope.set_const(
            &name,
            Rc::new(InterpreterValue::NativeFunction {
                name: name.clone(),
                body: function,
            }),
            0,
            0,
        )?;
    }

    for (name, function) in macros {
        interpreter.top_scope.set_const(
            &name,
            Rc::new(InterpreterValue::NativeMacro {
                name: name.clone(),
                body: function,
            }),
            0,
            0,
        )?;
    }

    interpreter.run_top_level()?;

    let main = interpreter.find_main()?;

    let result = interpreter.top_scope.evaluate_block(&main)?;

    Ok(result)
}