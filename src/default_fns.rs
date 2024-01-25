use std::{collections::HashMap, rc::Rc};

use crate::interpreter::{InterpreterError, InterpreterValue, NativeFn};

macro_rules! create_function {
    ($op:ident, $op2:ident, $op_str:expr) => {
        |scope, params| {
            if params.len() == 0 {
                return Err(InterpreterError::InvalidFunctionCall($op_str.to_owned()).into());
            }
            let params = scope.evaluate_each(params)?;

            let mut iter = params.into_iter();
            let first = iter.next().unwrap();
            let mut i = 0;
            let accum = iter.try_fold(first.clone(), |accum, next| {
                i += 1;
                match (accum.as_ref(), next.as_ref()) {
                    (InterpreterValue::Int(a), InterpreterValue::Int(b)) => {
                        return Ok(Rc::new(InterpreterValue::Int((std::ops::$op::$op2)(a, b))));
                    }
                    (InterpreterValue::Float(a), InterpreterValue::Float(b)) => {
                        return Ok(Rc::new(InterpreterValue::Float((std::ops::$op::$op2)(
                            a, b,
                        ))));
                    }
                    (_, b) => {
                        return Err(b.get_type().to_string());
                    }
                };
            });
            match accum {
                Err(e) => {
                    return Err(InterpreterError::InvalidTypeArgNative(
                        e,
                        i,
                        $op_str.to_owned(),
                        first.get_type().to_string(),
                    )
                    .into());
                }
                Ok(v) => return Ok(v),
            }
        }
    };
}

macro_rules! create_conversion_function {
    ($fn_name:expr, $return_type:ident, $int_conversion:expr, $float_conversion:expr, $string_conversion:expr, $bool_conversion:expr) => {
        |scope, params| {
            if params.len() != 1 {
                return Err(InterpreterError::InvalidFunctionCall($fn_name.to_owned()).into());
            }
            let params = scope.evaluate_each(params)?;

            let param = params.into_iter().next().unwrap();
            match param.as_ref() {
                InterpreterValue::Int(i) => {
                    Ok(Rc::new(InterpreterValue::$return_type($int_conversion(*i))))
                }
                InterpreterValue::Float(f) => Ok(Rc::new(InterpreterValue::$return_type(
                    $float_conversion(*f),
                ))),
                InterpreterValue::String(s) => Ok(Rc::new(InterpreterValue::$return_type(
                    $string_conversion(s),
                ))),
                InterpreterValue::Bool(b) => Ok(Rc::new(InterpreterValue::$return_type(
                    $bool_conversion(*b),
                ))),
                _ => Err(InterpreterError::InvalidType1Native(
                    param.get_type().to_string(),
                    $fn_name.to_owned(),
                )
                .into()),
            }
        }
    };
}

pub fn default_native_functions() -> HashMap<String, NativeFn> {
    let mut functions: HashMap<String, NativeFn> = HashMap::new();

    functions.insert("print".to_string(), |scope, params| {
        let params = scope.evaluate_each(params)?;
        for param in params {
            println!("{}", param.to_string());
        }
        Ok(Rc::new(InterpreterValue::Void))
    });

    functions.insert("dbg".to_string(), |scope, params| {
        if params.len() != 1 {
            return Err(InterpreterError::InvalidFunctionCall("dbg".to_owned()).into());
        }
        let params = scope.evaluate_each(params)?;
        println!("{:#?}", params[0]);
        Ok(params[0].clone())
    });

    functions.insert("==".to_string(), |scope, params| {
        if params.len() != 2 {
            return Err(InterpreterError::InvalidFunctionCall("==".to_owned()).into());
        }
        let params = scope.evaluate_each(params)?;

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

    functions.insert("!=".to_string(), |scope, params| {
        if params.len() != 2 {
            return Err(InterpreterError::InvalidFunctionCall("!=".to_owned()).into());
        }
        let params = scope.evaluate_each(params)?;

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

    functions.insert("+".to_string(), |scope, params| {
        if params.len() == 0 {
            return Err(InterpreterError::InvalidFunctionCall("+".to_owned()).into());
        }
        let params = scope.evaluate_each(params)?;

        let mut iter = params.into_iter();
        let first = iter.next().unwrap();
        let mut i = 0;
        let accum = iter.try_fold(first.clone(), |accum, next| {
            i += 1;
            match (accum.as_ref(), next.as_ref()) {
                (InterpreterValue::Int(a), InterpreterValue::Int(b)) => {
                    return Ok(Rc::new(InterpreterValue::Int((std::ops::Add::add)(a, b))));
                }
                (InterpreterValue::Float(a), InterpreterValue::Float(b)) => {
                    return Ok(Rc::new(InterpreterValue::Float((std::ops::Add::add)(a, b))));
                }
                (InterpreterValue::String(a), InterpreterValue::String(b)) => {
                    return Ok(Rc::new(InterpreterValue::String(a.to_owned() + b)));
                }
                (_, b) => {
                    return Err(b.get_type().to_string());
                }
            };
        });
        match accum {
            Err(e) => {
                return Err(InterpreterError::InvalidTypeArgNative(
                    e,
                    i,
                    "+".to_owned(),
                    first.get_type().to_string(),
                )
                .into());
            }
            Ok(v) => return Ok(v),
        }
    });

    functions.insert("-".to_string(), create_function!(Sub, sub, "-"));
    functions.insert("*".to_string(), create_function!(Mul, mul, "*"));
    functions.insert("/".to_string(), create_function!(Div, div, "/"));

    functions.insert(
        "int".to_string(),
        create_conversion_function!(
            "int",
            Int,
            |i| i,
            |f| f as i64,
            |s: &String| s.parse::<i64>().unwrap(),
            |b| b as i64
        ),
    );

    functions.insert(
        "float".to_string(),
        create_conversion_function!(
            "float",
            Float,
            |i| i as f64,
            |f| f as f64,
            |s: &String| s.parse::<f64>().unwrap(),
            |b| b as i64 as f64
        ),
    );

    functions.insert(
        "string".to_string(),
        create_conversion_function!(
            "string",
            String,
            |i: i64| i.to_string(),
            |f: f64| f.to_string(),
            |s: &String| s.to_string(),
            |b: bool| b.to_string()
        ),
    );

    functions.insert(
        "bool".to_string(),
        create_conversion_function!(
            "bool",
            Bool,
            |i| i != 0,
            |f| f != 0.0,
            |s: &String| s.parse().unwrap_or_default(),
            |b| b
        ),
    );

    functions
}
