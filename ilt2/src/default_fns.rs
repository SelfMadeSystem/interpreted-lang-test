use std::{collections::HashMap, rc::Rc};

use crate::{
    ast::{AstNode, AstNodeType},
    interpreter::{InterpreterError, InterpreterType, InterpreterValue, NativeFn, NativeMacro},
    token::TokenIdent,
};

pub fn native_functions() -> HashMap<String, NativeFn> {
    let mut functions: HashMap<String, NativeFn> = HashMap::new();

    functions.insert("print".to_string(), |_, args, _, _| {
        for arg in args {
            println!("{}", arg.to_string());
        }
        Ok(Rc::new(InterpreterValue::Void))
    });

    functions.insert("concat".to_string(), |_, args, _, _| {
        let mut result = String::new();
        for arg in args {
            result.push_str(&arg.to_string());
        }
        Ok(Rc::new(InterpreterValue::String(result)))
    });

    functions.insert("gettype".to_string(), |_, args, _, _| {
        if args.len() != 1 {
            return Err(InterpreterError::InvalidFunctionCall("gettype".to_owned()).into());
        }

        let arg = &args[0];

        Ok(Rc::new(InterpreterValue::Type(arg.get_type())))
    });

    functions.insert("matches".to_string(), |_, args, _, _| {
        if args.len() != 2 {
            return Err(InterpreterError::InvalidFunctionCall("matches".to_owned()).into());
        }

        let ty = match &args[0].as_ref() {
            InterpreterValue::Type(t) => t,
            _ => return Err(InterpreterError::InvalidFunctionCall("matches".to_owned()).into()),
        };

        let value = &args[1];

        Ok(Rc::new(InterpreterValue::Bool(value.check_type(ty))))
    });

    functions
}

pub fn native_macros() -> HashMap<String, NativeMacro> {
    let mut macros: HashMap<String, NativeMacro> = HashMap::new();

    macros.insert("fn".to_string(), |scope, args, line, col| {
        if args.len() < 2 {
            return Err(InterpreterError::InvalidMacroCall("fn".to_owned()).into());
        }

        let name = match &args[0].ty {
            AstNodeType::Ident(t) if matches!(t, TokenIdent::Ident(_)) => t,
            _ => return Err(InterpreterError::InvalidMacroCall("fn".to_owned()).into()),
        };

        let mut params = Vec::new();

        match &args[1].ty {
            AstNodeType::Array(params_) => {
                let mut iter = params_.into_iter().peekable();
                while let Some(param) = iter.next() {
                    match &param.ty {
                        AstNodeType::Ident(TokenIdent::Ident(s)) => params.push((
                            s.to_owned(),
                            match iter.peek() {
                                Some(AstNode {
                                    ty: AstNodeType::Ident(t),
                                    line,
                                    col,
                                }) if matches!(t, TokenIdent::Type(_)) => {
                                    match scope.get(t, *line, *col)?.as_ref() {
                                        InterpreterValue::Type(t) => {
                                            iter.next();
                                            t.clone()
                                        }
                                        _ => {
                                            return Err(InterpreterError::InvalidMacroCall(
                                                "fn".to_owned(),
                                            )
                                            .into())
                                        }
                                    }
                                }
                                _ => InterpreterType::Any,
                            },
                        )),
                        _ => return Err(InterpreterError::InvalidMacroCall("fn".to_owned()).into()),
                    }
                }
            }
            _ => return Err(InterpreterError::InvalidMacroCall("fn".to_owned()).into()),
        }

        let (return_type, has) = match &args[2].ty {
            AstNodeType::Ident(t) if matches!(t, TokenIdent::Type(_)) => {
                match scope.get(t, line, col)?.as_ref() {
                    InterpreterValue::Type(t) => (t.clone(), true),
                    _ => return Err(InterpreterError::InvalidMacroCall("fn".to_owned()).into()),
                }
            }
            _ => (InterpreterType::Any, false),
        };

        let body = args[if has { 3 } else { 2 }..].to_vec();

        let func = Rc::new(InterpreterValue::Function {
            name: name.to_string(),
            params,
            return_type,
            body,
        });

        if scope.top_scope {
            scope.set_const(name, func.clone(), line, col)?;

            Ok(Rc::new(InterpreterValue::Void))
        } else {
            Ok(func)
        }
    });

    macros.insert("macro".to_string(), |scope, args, line, col| {
        if args.len() < 2 {
            return Err(InterpreterError::InvalidMacroCall("macro".to_owned()).into());
        }

        let name = match &args[0].ty {
            AstNodeType::Ident(t) if matches!(t, TokenIdent::Ident(_)) => t,
            _ => return Err(InterpreterError::InvalidMacroCall("macro".to_owned()).into()),
        };

        let mut params = Vec::new();

        match &args[1].ty {
            AstNodeType::Array(params_) => {
                for param in params_ {
                    match &param.ty {
                        AstNodeType::Ident(TokenIdent::Ident(s)) => params.push(s.to_owned()),
                        _ => {
                            return Err(
                                InterpreterError::InvalidMacroCall("macro".to_owned()).into()
                            )
                        }
                    }
                }
            }
            _ => return Err(InterpreterError::InvalidMacroCall("macro".to_owned()).into()),
        }

        if params.len() != 2 {
            return Err(InterpreterError::InvalidMacroCall("macro".to_owned()).into());
        }

        let body = args[2..].to_vec();

        let func = Rc::new(InterpreterValue::Macro {
            name: name.to_string(),
            params,
            body,
        });

        if scope.top_scope {
            scope.set_const(name, func.clone(), line, col)?;

            Ok(Rc::new(InterpreterValue::Void))
        } else {
            Ok(func)
        }
    });

    macros.insert("call".to_string(), |scope, args, line, col| {
        if args.len() != 2 {
            return Err(InterpreterError::InvalidMacroCall("call".to_owned()).into());
        }

        let name = match &args[0].ty {
            AstNodeType::Ident(s) => s,
            _ => return Err(InterpreterError::InvalidMacroCall("call".to_owned()).into()),
        };

        let func = scope.get(name, line, col)?;

        let params = match &args[1].ty {
            AstNodeType::Array(args) => args,
            _ => return Err(InterpreterError::InvalidMacroCall("call".to_owned()).into()),
        };
        let params = scope.evaluate_each(params)?;

        Ok(scope.call_function(name, func, params, line, col)?)
    });

    macros.insert("ifelse".to_string(), |scope, args, line, col| {
        if args.len() != 3 {
            return Err(InterpreterError::InvalidMacroCall("ifelse".to_owned()).into());
        }

        let condition = match &args[0].ty {
            AstNodeType::Ident(s) => scope.get(s, line, col)?,
            AstNodeType::Bool(b) => Rc::new(InterpreterValue::Bool(*b)),
            AstNodeType::Call { .. } => scope.evaluate(&args[0])?,
            _ => return Err(InterpreterError::InvalidMacroCall("ifelse".to_owned()).into()),
        };

        let condition = match condition.as_ref() {
            InterpreterValue::Bool(b) => *b,
            _ => return Err(InterpreterError::InvalidMacroCall("ifelse".to_owned()).into()),
        };

        let body = if condition {
            &args[1]
        } else {
            &args[2]
        };

        scope.evaluate(body)
    });

    macros
}
