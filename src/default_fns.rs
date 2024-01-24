use std::{collections::HashMap, rc::Rc};

use crate::interpreter::{InterpreterError, InterpreterValue, NativeFn};

macro_rules! number_operation {
    ($op:expr, $a:expr, $b:expr) => {
        match ($a.as_ref(), $b.as_ref()) {
            (InterpreterValue::Int(a), InterpreterValue::Int(b)) => {
                return Ok(Rc::new(InterpreterValue::Int($op(a, b))));
            }
            (InterpreterValue::Float(a), InterpreterValue::Float(b)) => {
                return Ok(Rc::new(InterpreterValue::Float($op(a, b))));
            }
            (a, b) => {
                return Err(InterpreterError::InvalidType2Native(
                    a.get_type().to_string(),
                    b.get_type().to_string(),
                    stringify!($op).to_owned(),
                )
                .into());
            }
        }
    };
}

pub fn default_native_functions() -> HashMap<String, NativeFn> {
    let mut functions: HashMap<String, NativeFn> = HashMap::new();

    functions.insert("print".to_string(), |_, params| {
        for param in params {
            println!("{}", param.to_string());
        }
        Ok(Rc::new(InterpreterValue::Void))
    });

    functions.insert("dbg".to_string(), |_, params| {
        if params.len() != 1 {
            return Err(InterpreterError::InvalidFunctionCall("dbg".to_owned()).into());
        }
        println!("{:#?}", params[0]);
        Ok(params[0].clone())
    });

    functions.insert("==".to_string(), |_, params| {
        if params.len() != 2 {
            return Err(InterpreterError::InvalidFunctionCall("==".to_owned()).into());
        }

        let mut iter = params.into_iter();
        let first = iter.next().unwrap();
        for param in iter {
            match (first.as_ref(), param.as_ref()) {
                (InterpreterValue::Int(a), InterpreterValue::Int(b)) => {
                    return Ok(Rc::new(InterpreterValue::Bool(a == b)));
                }
                (InterpreterValue::Float(a), InterpreterValue::Float(b)) => {
                    return Ok(Rc::new(InterpreterValue::Bool(a == b)));
                }
                (InterpreterValue::String(a), InterpreterValue::String(b)) => {
                    return Ok(Rc::new(InterpreterValue::Bool(a == b)));
                }
                (InterpreterValue::Bool(a), InterpreterValue::Bool(b)) => {
                    return Ok(Rc::new(InterpreterValue::Bool(a == b)));
                }
                (a, b) => {
                    return Err(InterpreterError::InvalidType2Native(
                        a.get_type().to_string(),
                        b.get_type().to_string(),
                        "==".to_owned(),
                    )
                    .into());
                }
            }
        }
        Ok(Rc::new(InterpreterValue::Int(1)))
    });

    functions.insert("!=".to_string(), |_, params| {
        if params.len() != 2 {
            return Err(InterpreterError::InvalidFunctionCall("!=".to_owned()).into());
        }

        let mut iter = params.into_iter();
        let first = iter.next().unwrap();
        for param in iter {
            match (first.as_ref(), param.as_ref()) {
                (InterpreterValue::Int(a), InterpreterValue::Int(b)) => {
                    return Ok(Rc::new(InterpreterValue::Bool(a != b)));
                }
                (InterpreterValue::Float(a), InterpreterValue::Float(b)) => {
                    return Ok(Rc::new(InterpreterValue::Bool(a != b)));
                }
                (InterpreterValue::String(a), InterpreterValue::String(b)) => {
                    return Ok(Rc::new(InterpreterValue::Bool(a != b)));
                }
                (InterpreterValue::Bool(a), InterpreterValue::Bool(b)) => {
                    return Ok(Rc::new(InterpreterValue::Bool(a != b)));
                }
                (a, b) => {
                    return Err(InterpreterError::InvalidType2Native(
                        a.get_type().to_string(),
                        b.get_type().to_string(),
                        "!=".to_owned(),
                    )
                    .into());
                }
            }
        }
        Ok(Rc::new(InterpreterValue::Int(1)))
    });

    functions.insert("+".to_string(), |_, params| {
        if params.len() != 2 {
            return Err(InterpreterError::InvalidFunctionCall("+".to_owned()).into());
        }

        let mut iter = params.into_iter();
        let first = iter.next().unwrap();
        for param in iter {
            match (first.as_ref(), param.as_ref()) {
                (InterpreterValue::Int(a), InterpreterValue::Int(b)) => {
                    return Ok(Rc::new(InterpreterValue::Int((std::ops::Add::add)(a, b))));
                }
                (InterpreterValue::Float(a), InterpreterValue::Float(b)) => {
                    return Ok(Rc::new(InterpreterValue::Float((std::ops::Add::add)(a, b))));
                }
                (InterpreterValue::String(a), InterpreterValue::String(b)) => {
                    return Ok(Rc::new(InterpreterValue::String(a.to_owned() + b)));
                }
                (a, b) => {
                    return Err(InterpreterError::InvalidType2Native(
                        a.get_type().to_string(),
                        b.get_type().to_string(),
                        stringify!((std::ops::Add::add)).to_owned(),
                    )
                    .into());
                }
            };
        }
        Ok(Rc::new(InterpreterValue::Int(1)))
    });

    functions.insert("-".to_string(), |_, params| {
        if params.len() != 2 {
            return Err(InterpreterError::InvalidFunctionCall("-".to_owned()).into());
        }

        let mut iter = params.into_iter();
        let first = iter.next().unwrap();
        for param in iter {
            number_operation!(std::ops::Sub::sub, first, param);
        }
        Ok(Rc::new(InterpreterValue::Int(1)))
    });

    functions.insert("*".to_string(), |_, params| {
        if params.len() != 2 {
            return Err(InterpreterError::InvalidFunctionCall("*".to_owned()).into());
        }

        let mut iter = params.into_iter();
        let first = iter.next().unwrap();
        for param in iter {
            number_operation!(std::ops::Mul::mul, first, param);
        }
        Ok(Rc::new(InterpreterValue::Int(1)))
    });

    functions.insert("/".to_string(), |_, params| {
        if params.len() != 2 {
            return Err(InterpreterError::InvalidFunctionCall("/".to_owned()).into());
        }

        let mut iter = params.into_iter();
        let first = iter.next().unwrap();
        for param in iter {
            number_operation!(std::ops::Div::div, first, param);
        }
        Ok(Rc::new(InterpreterValue::Int(1)))
    });

    functions.insert("int".to_string(), |_, params| {
        if params.len() != 1 {
            return Err(InterpreterError::InvalidFunctionCall("int".to_owned()).into());
        }

        let param = params.into_iter().next().unwrap();
        match param.as_ref() {
            InterpreterValue::Int(i) => Ok(Rc::new(InterpreterValue::Int(*i))),
            InterpreterValue::Float(f) => Ok(Rc::new(InterpreterValue::Int(*f as i64))),
            InterpreterValue::String(s) => Ok(Rc::new(InterpreterValue::Int(s.parse().unwrap()))),
            InterpreterValue::Bool(b) => Ok(Rc::new(InterpreterValue::Int(*b as i64))),
            _ => Err(InterpreterError::InvalidType1Native(
                param.get_type().to_string(),
                "int".to_owned(),
            )
            .into()),
        }
    });

    functions.insert("float".to_string(), |_, params| {
        if params.len() != 1 {
            return Err(InterpreterError::InvalidFunctionCall("float".to_owned()).into());
        }

        let param = params.into_iter().next().unwrap();
        match param.as_ref() {
            InterpreterValue::Int(i) => Ok(Rc::new(InterpreterValue::Float(*i as f64))),
            InterpreterValue::Float(f) => Ok(Rc::new(InterpreterValue::Float(*f))),
            InterpreterValue::String(s) => Ok(Rc::new(InterpreterValue::Float(s.parse().unwrap()))),
            InterpreterValue::Bool(b) => Ok(Rc::new(InterpreterValue::Float(*b as i64 as f64))),
            _ => Err(InterpreterError::InvalidType1Native(
                param.get_type().to_string(),
                "float".to_owned(),
            )
            .into()),
        }
    });

    functions.insert("string".to_string(), |_, params| {
        if params.len() != 1 {
            return Err(InterpreterError::InvalidFunctionCall("string".to_owned()).into());
        }

        let param = params.into_iter().next().unwrap();
        match param.as_ref() {
            InterpreterValue::Int(i) => Ok(Rc::new(InterpreterValue::String(i.to_string()))),
            InterpreterValue::Float(f) => Ok(Rc::new(InterpreterValue::String(f.to_string()))),
            InterpreterValue::String(s) => Ok(Rc::new(InterpreterValue::String(s.to_string()))),
            InterpreterValue::Bool(b) => Ok(Rc::new(InterpreterValue::String(b.to_string()))),
            _ => Err(InterpreterError::InvalidType1Native(
                param.get_type().to_string(),
                "string".to_owned(),
            )
            .into()),
        }
    });

    functions.insert("bool".to_string(), |_, params| {
        if params.len() != 1 {
            return Err(InterpreterError::InvalidFunctionCall("bool".to_owned()).into());
        }

        let param = params.into_iter().next().unwrap();
        match param.as_ref() {
            InterpreterValue::Int(i) => Ok(Rc::new(InterpreterValue::Bool(*i != 0))),
            InterpreterValue::Float(f) => Ok(Rc::new(InterpreterValue::Bool(*f != 0.0))),
            InterpreterValue::String(s) => Ok(Rc::new(InterpreterValue::Bool(!s.is_empty()))),
            InterpreterValue::Bool(b) => Ok(Rc::new(InterpreterValue::Bool(*b))),
            _ => Err(InterpreterError::InvalidType1Native(
                param.get_type().to_string(),
                "bool".to_owned(),
            )
            .into()),
        }
    });

    functions.insert("if".to_string(), |scope, params| {
        if params.len() < 2 || params.len() > 3 {
            return Err(InterpreterError::InvalidFunctionCall("if".to_owned()).into());
        }

        let mut iter = params.into_iter();
        let condition = iter.next().unwrap();
        let if_true = iter.next().unwrap();
        let if_false = iter.next();

        let condition = match condition.as_ref() {
            InterpreterValue::Bool(b) => *b,
            _ => {
                return Err(InterpreterError::InvalidType1Native(
                    condition.get_type().to_string(),
                    "if".to_owned(),
                )
                .into());
            }
        };

        if condition {
            match if_true.as_ref() {
                InterpreterValue::Function { params, body, .. } => {
                    let mut scope = scope.new_child();
                    for param in params.iter() {
                        scope.set(param, Rc::new(InterpreterValue::Void))?;
                    }
                    return Ok(scope.evaluate_block(body)?);
                }
                _ => return Ok(if_true),
            }
        } else if let Some(if_false) = if_false {
            match if_false.as_ref() {
                InterpreterValue::Function { params, body, .. } => {
                    let mut scope = scope.new_child();
                    for param in params.iter() {
                        scope.set(param, Rc::new(InterpreterValue::Void))?;
                    }
                    return Ok(scope.evaluate_block(body)?);
                }
                _ => return Ok(if_false),
            }
        } else {
            return Ok(Rc::new(InterpreterValue::Void));
        }
    });

    functions.insert("while".to_string(), |scope, params| {
        if params.len() != 2 {
            return Err(InterpreterError::InvalidFunctionCall("if".to_owned()).into());
        }

        let mut iter = params.into_iter();
        let condition = iter.next().unwrap();
        let body = iter.next().unwrap();

        let mut value = Rc::new(InterpreterValue::Void);

        loop {
            let condition = match condition.as_ref() {
                InterpreterValue::Bool(b) => *b,
                InterpreterValue::Function { params, body, .. } => {
                    if params.len() != 0 {
                        return Err(
                            InterpreterError::InvalidFunctionCall("while".to_owned()).into()
                        );
                    }
                    let mut scope = scope.new_child();
                    let result = scope.evaluate_block(body)?;
                    match result.as_ref() {
                        InterpreterValue::Bool(b) => *b,
                        _ => {
                            return Err(InterpreterError::InvalidType1Native(
                                result.get_type().to_string(),
                                "while".to_owned(),
                            )
                            .into());
                        }
                    }
                }
                _ => {
                    return Err(InterpreterError::InvalidType1Native(
                        condition.get_type().to_string(),
                        "while".to_owned(),
                    )
                    .into());
                }
            };
            let mut scope = scope.new_child();

            if condition {
                match body.as_ref() {
                    InterpreterValue::Function { params, body, .. } => {
                        for param in params.iter() {
                            scope.set(param, Rc::new(InterpreterValue::Void))?;
                        }
                        value = scope.evaluate_block(body)?;
                    }
                    _ => return Ok(body),
                }
            } else {
                break Ok(value);
            }
        }
    });

    functions
}
