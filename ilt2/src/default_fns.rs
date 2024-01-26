use std::{collections::HashMap, rc::Rc};

use crate::{
    ast::AstNodeType,
    interpreter::{InterpreterError, InterpreterValue, NativeFn, NativeMacro},
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

    functions
}

pub fn native_macros() -> HashMap<String, NativeMacro> {
    let mut macros: HashMap<String, NativeMacro> = HashMap::new();

    macros.insert("fn".to_string(), |scope, args, line, col| {
        if args.len() < 2 {
            return Err(InterpreterError::InvalidMacroCall("fn".to_owned()).into());
        }

        let name = match &args[0].ty {
            AstNodeType::Ident(TokenIdent::Ident(s)) => s,
            _ => return Err(InterpreterError::InvalidMacroCall("fn".to_owned()).into()),
        };

        let mut params = Vec::new();

        match &args[1].ty {
            AstNodeType::Array(params_) => {
                for param in params_ {
                    match &param.ty {
                        AstNodeType::Ident(TokenIdent::Ident(s)) => params.push(s.to_owned()),
                        _ => return Err(InterpreterError::InvalidMacroCall("fn".to_owned()).into()),
                    }
                }
            }
            _ => return Err(InterpreterError::InvalidMacroCall("fn".to_owned()).into()),
        }

        let body = args[2..].to_vec();

        let func = Rc::new(InterpreterValue::Function {
            name: name.to_owned(),
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

    macros.insert("macro".to_string(), |scope, args, line, col| {
        if args.len() < 2 {
            return Err(InterpreterError::InvalidMacroCall("macro".to_owned()).into());
        }

        let name = match &args[0].ty {
            AstNodeType::Ident(TokenIdent::Ident(s)) => s,
            _ => return Err(InterpreterError::InvalidMacroCall("macro".to_owned()).into()),
        };

        let mut params = Vec::new();

        match &args[1].ty {
            AstNodeType::Array(params_) => {
                for param in params_ {
                    match &param.ty {
                        AstNodeType::Ident(TokenIdent::Ident(s)) => params.push(s.to_owned()),
                        _ => return Err(InterpreterError::InvalidMacroCall("macro".to_owned()).into()),
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
            name: name.to_owned(),
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
            AstNodeType::Ident(s) => s.as_str(),
            _ => return Err(InterpreterError::InvalidMacroCall("call".to_owned()).into()),
        };

        let params = match &args[1].ty {
            AstNodeType::Array(args) => args,
            _ => return Err(InterpreterError::InvalidMacroCall("call".to_owned()).into()),
        };
        let params = scope.evaluate_each(params)?;

        Ok(scope.call_function(name, params, line, col)?)
    });

    macros
}
