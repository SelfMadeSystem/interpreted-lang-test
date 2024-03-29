use std::{cell::RefCell, collections::HashMap, rc::Rc};

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

    functions.insert("gettype".to_string(), |_, args, _, _| {
        if args.len() != 1 {
            return Err(InterpreterError::InvalidFunctionCall("gettype".to_owned()).into());
        }

        let arg = &args[0];

        Ok(Rc::new(InterpreterValue::Type(arg.get_type())))
    });

    // returns true if value 2 is of type value 1 (value 1 is a type)
    functions.insert("istype".to_string(), |_, args, _, _| {
        if args.len() != 2 {
            return Err(InterpreterError::InvalidFunctionCall("istype".to_owned()).into());
        }

        let ty = match &args[0].as_ref() {
            InterpreterValue::Type(t) => t,
            _ => return Err(InterpreterError::InvalidFunctionCall("istype".to_owned()).into()),
        };

        let value = &args[1];

        Ok(Rc::new(InterpreterValue::Bool(value.check_type(ty))))
    });

    // returns true if value 1 is assignable to type value 2 (both are types)
    functions.insert("isassignable".to_string(), |_, args, _, _| {
        if args.len() != 2 {
            return Err(InterpreterError::InvalidFunctionCall("isassignable".to_owned()).into());
        }

        let ty = match &args[0].as_ref() {
            InterpreterValue::Type(t) => t,
            _ => {
                return Err(InterpreterError::InvalidFunctionCall("isassignable".to_owned()).into())
            }
        };

        let value = match &args[1].as_ref() {
            InterpreterValue::Type(t) => t,
            _ => {
                return Err(InterpreterError::InvalidFunctionCall("isassignable".to_owned()).into())
            }
        };

        Ok(Rc::new(InterpreterValue::Bool(ty.is_assignable(value))))
    });

    functions.insert("as".to_string(), |_, args, _, _| {
        if args.len() != 2 {
            return Err(InterpreterError::InvalidFunctionCall("as".to_owned()).into());
        }

        let ty = match &args[0].as_ref() {
            InterpreterValue::Type(t) => t,
            _ => return Err(InterpreterError::InvalidFunctionCall("as".to_owned()).into()),
        };

        let value = &args[1];

        Ok(Rc::new(value.as_type(ty)?))
    });

    string_functions(&mut functions);
    comparison_functions(&mut functions);
    math_functions(&mut functions);
    array_functions(&mut functions);

    functions
}

fn string_functions(functions: &mut HashMap<String, NativeFn>) {
    functions.insert("concat".to_string(), |_, args, _, _| {
        let mut result = String::new();
        for arg in args {
            result.push_str(&arg.to_string());
        }
        Ok(Rc::new(InterpreterValue::String(result)))
    });
}

fn comparison_functions(functions: &mut HashMap<String, NativeFn>) {
    functions.insert("==".to_string(), |_, args, _, _| {
        if args.len() != 2 {
            return Err(InterpreterError::InvalidFunctionCall("==".to_owned()).into());
        }

        let left = &args[0];
        let right = &args[1];

        Ok(Rc::new(InterpreterValue::Bool(left == right)))
    });

    functions.insert("!=".to_string(), |_, args, _, _| {
        if args.len() != 2 {
            return Err(InterpreterError::InvalidFunctionCall("!=".to_owned()).into());
        }

        let left = &args[0];
        let right = &args[1];

        Ok(Rc::new(InterpreterValue::Bool(left != right)))
    });

    functions.insert("<".to_string(), |_, args, _, _| {
        if args.len() != 2 {
            return Err(InterpreterError::InvalidFunctionCall("<".to_owned()).into());
        }

        let left = &args[0];
        let right = &args[1];

        match (left.as_ref(), right.as_ref()) {
            (InterpreterValue::Int(l), InterpreterValue::Int(r)) => {
                Ok(Rc::new(InterpreterValue::Bool(l < r)))
            }
            (InterpreterValue::Float(l), InterpreterValue::Float(r)) => {
                Ok(Rc::new(InterpreterValue::Bool(l < r)))
            }
            (InterpreterValue::String(l), InterpreterValue::String(r)) => {
                Ok(Rc::new(InterpreterValue::Bool(l < r)))
            }
            _ => Err(InterpreterError::InvalidFunctionCall("<".to_owned()).into()),
        }
    });

    functions.insert("<=".to_string(), |_, args, _, _| {
        if args.len() != 2 {
            return Err(InterpreterError::InvalidFunctionCall("<=".to_owned()).into());
        }

        let left = &args[0];
        let right = &args[1];

        match (left.as_ref(), right.as_ref()) {
            (InterpreterValue::Int(l), InterpreterValue::Int(r)) => {
                Ok(Rc::new(InterpreterValue::Bool(l <= r)))
            }
            (InterpreterValue::Float(l), InterpreterValue::Float(r)) => {
                Ok(Rc::new(InterpreterValue::Bool(l <= r)))
            }
            (InterpreterValue::String(l), InterpreterValue::String(r)) => {
                Ok(Rc::new(InterpreterValue::Bool(l <= r)))
            }
            _ => Err(InterpreterError::InvalidFunctionCall("<=".to_owned()).into()),
        }
    });

    functions.insert(">".to_string(), |_, args, _, _| {
        if args.len() != 2 {
            return Err(InterpreterError::InvalidFunctionCall(">".to_owned()).into());
        }

        let left = &args[0];
        let right = &args[1];

        match (left.as_ref(), right.as_ref()) {
            (InterpreterValue::Int(l), InterpreterValue::Int(r)) => {
                Ok(Rc::new(InterpreterValue::Bool(l > r)))
            }
            (InterpreterValue::Float(l), InterpreterValue::Float(r)) => {
                Ok(Rc::new(InterpreterValue::Bool(l > r)))
            }
            (InterpreterValue::String(l), InterpreterValue::String(r)) => {
                Ok(Rc::new(InterpreterValue::Bool(l > r)))
            }
            _ => Err(InterpreterError::InvalidFunctionCall(">".to_owned()).into()),
        }
    });

    functions.insert(">=".to_string(), |_, args, _, _| {
        if args.len() != 2 {
            return Err(InterpreterError::InvalidFunctionCall(">=".to_owned()).into());
        }

        let left = &args[0];
        let right = &args[1];

        match (left.as_ref(), right.as_ref()) {
            (InterpreterValue::Int(l), InterpreterValue::Int(r)) => {
                Ok(Rc::new(InterpreterValue::Bool(l >= r)))
            }
            (InterpreterValue::Float(l), InterpreterValue::Float(r)) => {
                Ok(Rc::new(InterpreterValue::Bool(l >= r)))
            }
            (InterpreterValue::String(l), InterpreterValue::String(r)) => {
                Ok(Rc::new(InterpreterValue::Bool(l >= r)))
            }
            _ => Err(InterpreterError::InvalidFunctionCall(">=".to_owned()).into()),
        }
    });
}

fn math_functions(functions: &mut HashMap<String, NativeFn>) {
    functions.insert("+".to_string(), |_, args, _, _| {
        if args.len() == 0 {
            return Err(InterpreterError::InvalidFunctionCall("+".to_owned()).into());
        }

        let first = &args[0].as_ref();
        let mut result = match first {
            InterpreterValue::Int(_) => (**first).clone(),
            InterpreterValue::Float(_) => (**first).clone(),
            _ => return Err(InterpreterError::InvalidFunctionCall("+".to_owned()).into()),
        };

        for arg in &args[1..] {
            match (result, arg.as_ref()) {
                (InterpreterValue::Int(l), InterpreterValue::Int(r)) => {
                    result = InterpreterValue::Int(l + r)
                }
                (InterpreterValue::Float(l), InterpreterValue::Float(r)) => {
                    result = InterpreterValue::Float(l + r)
                }
                _ => return Err(InterpreterError::InvalidFunctionCall("+".to_owned()).into()),
            }
        }

        Ok(Rc::new(result))
    });

    functions.insert("-".to_string(), |_, args, _, _| {
        if args.len() != 2 {
            return Err(InterpreterError::InvalidFunctionCall("-".to_owned()).into());
        }

        let left = &args[0];
        let right = &args[1];

        match (left.as_ref(), right.as_ref()) {
            (InterpreterValue::Int(l), InterpreterValue::Int(r)) => {
                Ok(Rc::new(InterpreterValue::Int(l - r)))
            }
            (InterpreterValue::Float(l), InterpreterValue::Float(r)) => {
                Ok(Rc::new(InterpreterValue::Float(l - r)))
            }
            _ => Err(InterpreterError::InvalidFunctionCall("-".to_owned()).into()),
        }
    });

    functions.insert("*".to_string(), |_, args, _, _| {
        if args.len() == 0 {
            return Err(InterpreterError::InvalidFunctionCall("*".to_owned()).into());
        }

        let mut result = match &args[0].as_ref() {
            InterpreterValue::Int(i) => *i,
            InterpreterValue::Float(f) => *f as i64,
            _ => return Err(InterpreterError::InvalidFunctionCall("*".to_owned()).into()),
        };

        for arg in &args[1..] {
            match &arg.as_ref() {
                InterpreterValue::Int(i) => result *= *i,
                InterpreterValue::Float(f) => result *= *f as i64,
                _ => return Err(InterpreterError::InvalidFunctionCall("*".to_owned()).into()),
            }
        }

        Ok(Rc::new(InterpreterValue::Int(result)))
    });

    functions.insert("/".to_string(), |_, args, _, _| {
        if args.len() != 2 {
            return Err(InterpreterError::InvalidFunctionCall("/".to_owned()).into());
        }

        let left = &args[0];
        let right = &args[1];

        match (left.as_ref(), right.as_ref()) {
            (InterpreterValue::Int(l), InterpreterValue::Int(r)) => {
                Ok(Rc::new(InterpreterValue::Int(l / r)))
            }
            (InterpreterValue::Float(l), InterpreterValue::Float(r)) => {
                Ok(Rc::new(InterpreterValue::Float(l / r)))
            }
            _ => Err(InterpreterError::InvalidFunctionCall("/".to_owned()).into()),
        }
    });

    functions.insert("%".to_string(), |_, args, _, _| {
        if args.len() != 2 {
            return Err(InterpreterError::InvalidFunctionCall("%".to_owned()).into());
        }

        let left = &args[0];
        let right = &args[1];

        match (left.as_ref(), right.as_ref()) {
            (InterpreterValue::Int(l), InterpreterValue::Int(r)) => {
                Ok(Rc::new(InterpreterValue::Int(l % r)))
            }
            (InterpreterValue::Float(l), InterpreterValue::Float(r)) => {
                Ok(Rc::new(InterpreterValue::Float(l % r)))
            }
            _ => Err(InterpreterError::InvalidFunctionCall("%".to_owned()).into()),
        }
    });

    functions.insert("^".to_string(), |_, args, _, _| {
        if args.len() != 2 {
            return Err(InterpreterError::InvalidFunctionCall("^".to_owned()).into());
        }

        let left = &args[0];
        let right = &args[1];

        match (left.as_ref(), right.as_ref()) {
            (InterpreterValue::Int(l), InterpreterValue::Int(r)) => {
                Ok(Rc::new(InterpreterValue::Int(l.pow(*r as u32))))
            }
            (InterpreterValue::Float(l), InterpreterValue::Float(r)) => {
                Ok(Rc::new(InterpreterValue::Float(l.powf(*r))))
            }
            _ => Err(InterpreterError::InvalidFunctionCall("^".to_owned()).into()),
        }
    });

    functions.insert("sqrt".to_string(), |_, args, _, _| {
        if args.len() != 1 {
            return Err(InterpreterError::InvalidFunctionCall("sqrt".to_owned()).into());
        }

        let value = &args[0];

        match value.as_ref() {
            InterpreterValue::Int(v) => Ok(Rc::new(InterpreterValue::Float((*v as f64).sqrt()))),
            InterpreterValue::Float(v) => Ok(Rc::new(InterpreterValue::Float(v.sqrt()))),
            _ => Err(InterpreterError::InvalidFunctionCall("sqrt".to_owned()).into()),
        }
    });

    functions.insert("sin".to_string(), |_, args, _, _| {
        if args.len() != 1 {
            return Err(InterpreterError::InvalidFunctionCall("sin".to_owned()).into());
        }

        let value = &args[0];

        match value.as_ref() {
            InterpreterValue::Int(v) => Ok(Rc::new(InterpreterValue::Float((*v as f64).sin()))),
            InterpreterValue::Float(v) => Ok(Rc::new(InterpreterValue::Float(v.sin()))),
            _ => Err(InterpreterError::InvalidFunctionCall("sin".to_owned()).into()),
        }
    });

    functions.insert("cos".to_string(), |_, args, _, _| {
        if args.len() != 1 {
            return Err(InterpreterError::InvalidFunctionCall("cos".to_owned()).into());
        }

        let value = &args[0];

        match value.as_ref() {
            InterpreterValue::Int(v) => Ok(Rc::new(InterpreterValue::Float((*v as f64).cos()))),
            InterpreterValue::Float(v) => Ok(Rc::new(InterpreterValue::Float(v.cos()))),
            _ => Err(InterpreterError::InvalidFunctionCall("cos".to_owned()).into()),
        }
    });

    functions.insert("tan".to_string(), |_, args, _, _| {
        if args.len() != 1 {
            return Err(InterpreterError::InvalidFunctionCall("tan".to_owned()).into());
        }

        let value = &args[0];

        match value.as_ref() {
            InterpreterValue::Int(v) => Ok(Rc::new(InterpreterValue::Float((*v as f64).tan()))),
            InterpreterValue::Float(v) => Ok(Rc::new(InterpreterValue::Float(v.tan()))),
            _ => Err(InterpreterError::InvalidFunctionCall("tan".to_owned()).into()),
        }
    });

    functions.insert("asin".to_string(), |_, args, _, _| {
        if args.len() != 1 {
            return Err(InterpreterError::InvalidFunctionCall("asin".to_owned()).into());
        }

        let value = &args[0];

        match value.as_ref() {
            InterpreterValue::Int(v) => Ok(Rc::new(InterpreterValue::Float((*v as f64).asin()))),
            InterpreterValue::Float(v) => Ok(Rc::new(InterpreterValue::Float(v.asin()))),
            _ => Err(InterpreterError::InvalidFunctionCall("asin".to_owned()).into()),
        }
    });

    functions.insert("acos".to_string(), |_, args, _, _| {
        if args.len() != 1 {
            return Err(InterpreterError::InvalidFunctionCall("acos".to_owned()).into());
        }

        let value = &args[0];

        match value.as_ref() {
            InterpreterValue::Int(v) => Ok(Rc::new(InterpreterValue::Float((*v as f64).acos()))),
            InterpreterValue::Float(v) => Ok(Rc::new(InterpreterValue::Float(v.acos()))),
            _ => Err(InterpreterError::InvalidFunctionCall("acos".to_owned()).into()),
        }
    });

    functions.insert("atan".to_string(), |_, args, _, _| {
        if args.len() != 1 {
            return Err(InterpreterError::InvalidFunctionCall("atan".to_owned()).into());
        }

        let value = &args[0];

        match value.as_ref() {
            InterpreterValue::Int(v) => Ok(Rc::new(InterpreterValue::Float((*v as f64).atan()))),
            InterpreterValue::Float(v) => Ok(Rc::new(InterpreterValue::Float(v.atan()))),
            _ => Err(InterpreterError::InvalidFunctionCall("atan".to_owned()).into()),
        }
    });

    functions.insert("atan2".to_string(), |_, args, _, _| {
        if args.len() != 2 {
            return Err(InterpreterError::InvalidFunctionCall("atan2".to_owned()).into());
        }

        let left = &args[0];
        let right = &args[1];

        match (left.as_ref(), right.as_ref()) {
            (InterpreterValue::Int(l), InterpreterValue::Int(r)) => Ok(Rc::new(
                InterpreterValue::Float((*l as f64).atan2(*r as f64)),
            )),
            (InterpreterValue::Float(l), InterpreterValue::Float(r)) => {
                Ok(Rc::new(InterpreterValue::Float(l.atan2(*r))))
            }
            _ => Err(InterpreterError::InvalidFunctionCall("atan2".to_owned()).into()),
        }
    });

    functions.insert("ln".to_string(), |_, args, _, _| {
        if args.len() != 1 {
            return Err(InterpreterError::InvalidFunctionCall("ln".to_owned()).into());
        }

        let value = &args[0];

        match value.as_ref() {
            InterpreterValue::Int(v) => Ok(Rc::new(InterpreterValue::Float((*v as f64).ln()))),
            InterpreterValue::Float(v) => Ok(Rc::new(InterpreterValue::Float(v.ln()))),
            _ => Err(InterpreterError::InvalidFunctionCall("ln".to_owned()).into()),
        }
    });

    functions.insert("log".to_string(), |_, args, _, _| {
        if args.len() != 2 {
            return Err(InterpreterError::InvalidFunctionCall("log".to_owned()).into());
        }

        let value = &args[0];
        let base = &args[1];

        match (value.as_ref(), base.as_ref()) {
            (InterpreterValue::Int(v), InterpreterValue::Int(b)) => {
                Ok(Rc::new(InterpreterValue::Float((*v as f64).log(*b as f64))))
            }
            (InterpreterValue::Float(v), InterpreterValue::Float(b)) => {
                Ok(Rc::new(InterpreterValue::Float(v.log(*b))))
            }
            _ => Err(InterpreterError::InvalidFunctionCall("log".to_owned()).into()),
        }
    });

    functions.insert("floor".to_string(), |_, args, _, _| {
        if args.len() != 1 {
            return Err(InterpreterError::InvalidFunctionCall("floor".to_owned()).into());
        }

        let value = &args[0];

        match value.as_ref() {
            InterpreterValue::Int(v) => Ok(Rc::new(InterpreterValue::Int(*v))),
            InterpreterValue::Float(v) => Ok(Rc::new(InterpreterValue::Int(v.floor() as i64))),
            _ => Err(InterpreterError::InvalidFunctionCall("floor".to_owned()).into()),
        }
    });

    functions.insert("ceil".to_string(), |_, args, _, _| {
        if args.len() != 1 {
            return Err(InterpreterError::InvalidFunctionCall("ceil".to_owned()).into());
        }

        let value = &args[0];

        match value.as_ref() {
            InterpreterValue::Int(v) => Ok(Rc::new(InterpreterValue::Int(*v))),
            InterpreterValue::Float(v) => Ok(Rc::new(InterpreterValue::Int(v.ceil() as i64))),
            _ => Err(InterpreterError::InvalidFunctionCall("ceil".to_owned()).into()),
        }
    });

    functions.insert("round".to_string(), |_, args, _, _| {
        if args.len() != 1 {
            return Err(InterpreterError::InvalidFunctionCall("round".to_owned()).into());
        }

        let value = &args[0];

        match value.as_ref() {
            InterpreterValue::Int(v) => Ok(Rc::new(InterpreterValue::Int(*v))),
            InterpreterValue::Float(v) => Ok(Rc::new(InterpreterValue::Int(v.round() as i64))),
            _ => Err(InterpreterError::InvalidFunctionCall("round".to_owned()).into()),
        }
    });

    functions.insert("abs".to_string(), |_, args, _, _| {
        if args.len() != 1 {
            return Err(InterpreterError::InvalidFunctionCall("abs".to_owned()).into());
        }

        let value = &args[0];

        match value.as_ref() {
            InterpreterValue::Int(v) => Ok(Rc::new(InterpreterValue::Int(v.abs()))),
            InterpreterValue::Float(v) => Ok(Rc::new(InterpreterValue::Float(v.abs()))),
            _ => Err(InterpreterError::InvalidFunctionCall("abs".to_owned()).into()),
        }
    });

    functions.insert("max".to_string(), |_, args, _, _| {
        if args.len() == 0 {
            return Err(InterpreterError::InvalidFunctionCall("max".to_owned()).into());
        }

        let mut result = match &args[0].as_ref() {
            InterpreterValue::Int(i) => *i,
            InterpreterValue::Float(f) => *f as i64,
            _ => return Err(InterpreterError::InvalidFunctionCall("max".to_owned()).into()),
        };

        for arg in &args[1..] {
            match &arg.as_ref() {
                InterpreterValue::Int(i) => result = result.max(*i),
                InterpreterValue::Float(f) => result = result.max(*f as i64),
                _ => return Err(InterpreterError::InvalidFunctionCall("max".to_owned()).into()),
            }
        }

        Ok(Rc::new(InterpreterValue::Int(result)))
    });

    functions.insert("min".to_string(), |_, args, _, _| {
        if args.len() == 0 {
            return Err(InterpreterError::InvalidFunctionCall("min".to_owned()).into());
        }

        let mut result = match &args[0].as_ref() {
            InterpreterValue::Int(i) => *i,
            InterpreterValue::Float(f) => *f as i64,
            _ => return Err(InterpreterError::InvalidFunctionCall("min".to_owned()).into()),
        };

        for arg in &args[1..] {
            match &arg.as_ref() {
                InterpreterValue::Int(i) => result = result.min(*i),
                InterpreterValue::Float(f) => result = result.min(*f as i64),
                _ => return Err(InterpreterError::InvalidFunctionCall("min".to_owned()).into()),
            }
        }

        Ok(Rc::new(InterpreterValue::Int(result)))
    });
}

fn array_functions(functions: &mut HashMap<String, NativeFn>) {
    functions.insert("len".to_string(), |_, args, _, _| {
        if args.len() != 1 {
            return Err(InterpreterError::InvalidFunctionCall("len".to_owned()).into());
        }

        let value = &args[0];

        match value.as_ref() {
            InterpreterValue::Array(a) => Ok(Rc::new(InterpreterValue::Int(a.borrow().len() as i64))),
            _ => Err(InterpreterError::InvalidFunctionCall("len".to_owned()).into()),
        }
    });

    functions.insert("push".to_string(), |_, args, _, _| {
        if args.len() != 2 {
            return Err(InterpreterError::InvalidFunctionCall("push".to_owned()).into());
        }

        let array = &args[0];
        let value = &args[1];

        match array.as_ref() {
            InterpreterValue::Array(a) => {
                let mut a = a.borrow_mut();
                a.push(value.clone());
                Ok(Rc::new(InterpreterValue::Void))
            }
            _ => Err(InterpreterError::InvalidFunctionCall("push".to_owned()).into()),
        }
    });

    functions.insert("pop".to_string(), |_, args, _, _| {
        if args.len() != 1 {
            return Err(InterpreterError::InvalidFunctionCall("pop".to_owned()).into());
        }

        let array = &args[0];

        match array.as_ref() {
            InterpreterValue::Array(a) => {
                let mut a = a.borrow_mut();
                let value = a.pop().unwrap();
                Ok(value)
            }
            _ => Err(InterpreterError::InvalidFunctionCall("pop".to_owned()).into()),
        }
    });

    functions.insert("get".to_string(), |_, args, _, _| {
        if args.len() != 2 {
            return Err(InterpreterError::InvalidFunctionCall("get".to_owned()).into());
        }

        let array = &args[0];
        let index = &args[1];

        match (array.as_ref(), index.as_ref()) {
            (InterpreterValue::Array(a), InterpreterValue::Int(i)) => {
                let a = a.borrow();
                let i = *i as usize;
                if i >= a.len() {
                    return Err(InterpreterError::InvalidFunctionCall("get".to_owned()).into());
                }
                Ok(a[i].clone())
            }
            _ => Err(InterpreterError::InvalidFunctionCall("get".to_owned()).into()),
        }
    });

    functions.insert("set".to_string(), |_, args, _, _| {
        if args.len() != 3 {
            return Err(InterpreterError::InvalidFunctionCall("set".to_owned()).into());
        }

        let array = &args[0];
        let index = &args[1];
        let value = &args[2];

        match (array.as_ref(), index.as_ref()) {
            (InterpreterValue::Array(a), InterpreterValue::Int(i)) => {
                let mut a = a.borrow_mut();
                let i = *i as usize;
                if i >= a.len() {
                    return Err(InterpreterError::InvalidFunctionCall("set".to_owned()).into());
                }
                let prev = a[i].clone();
                a[i] = value.clone();
                Ok(prev)
            }
            _ => Err(InterpreterError::InvalidFunctionCall("set".to_owned()).into()),
        }
    });

    functions.insert("remove".to_string(), |_, args, _, _| {
        if args.len() != 2 {
            return Err(InterpreterError::InvalidFunctionCall("remove".to_owned()).into());
        }

        let array = &args[0];
        let index = &args[1];

        match (array.as_ref(), index.as_ref()) {
            (InterpreterValue::Array(a), InterpreterValue::Int(i)) => {
                let mut a = a.borrow_mut();
                let i = *i as usize;
                if i >= a.len() {
                    return Err(InterpreterError::InvalidFunctionCall("remove".to_owned()).into());
                }
                let value = a.remove(i);
                Ok(value)
            }
            _ => Err(InterpreterError::InvalidFunctionCall("remove".to_owned()).into()),
        }
    });

    functions.insert("insert".to_string(), |_, args, _, _| {
        if args.len() != 3 {
            return Err(InterpreterError::InvalidFunctionCall("insert".to_owned()).into());
        }

        let array = &args[0];
        let index = &args[1];
        let value = &args[2];

        match (array.as_ref(), index.as_ref()) {
            (InterpreterValue::Array(a), InterpreterValue::Int(i)) => {
                let mut a = a.borrow_mut();
                let i = *i as usize;
                if i > a.len() {
                    return Err(InterpreterError::InvalidFunctionCall("insert".to_owned()).into());
                }
                a.insert(i, value.clone());
                Ok(Rc::new(InterpreterValue::Void))
            }
            _ => Err(InterpreterError::InvalidFunctionCall("insert".to_owned()).into()),
        }
    });

    functions.insert("has".to_string(), |_, args, _, _| {
        if args.len() != 2 {
            return Err(InterpreterError::InvalidFunctionCall("has".to_owned()).into());
        }

        let array = &args[0];
        let value = &args[1];

        match array.as_ref() {
            InterpreterValue::Array(a) => Ok(Rc::new(InterpreterValue::Bool(
                a.borrow().iter().any(|v| v == value),
            ))),
            _ => Err(InterpreterError::InvalidFunctionCall("has".to_owned()).into()),
        }
    });

    functions.insert("head".to_string(), |_, args, _, _| {
        if args.len() != 1 {
            return Err(InterpreterError::InvalidFunctionCall("head".to_owned()).into());
        }

        let array = &args[0];

        match array.as_ref() {
            InterpreterValue::Array(a) => {
                let a = a.borrow();
                if a.len() == 0 {
                    return Err(InterpreterError::InvalidFunctionCall("head".to_owned()).into());
                }
                Ok(a[0].clone())
            }
            _ => Err(InterpreterError::InvalidFunctionCall("head".to_owned()).into()),
        }
    });

    functions.insert("tail".to_string(), |_, args, _, _| {
        if args.len() != 1 {
            return Err(InterpreterError::InvalidFunctionCall("tail".to_owned()).into());
        }

        let array = &args[0];

        match array.as_ref() {
            InterpreterValue::Array(a) => {
                let a = a.borrow();
                if a.len() == 0 {
                    return Err(InterpreterError::InvalidFunctionCall("tail".to_owned()).into());
                }
                Ok(Rc::new(InterpreterValue::Array(RefCell::new(
                    a[1..].to_vec(),
                ))))
            }
            _ => Err(InterpreterError::InvalidFunctionCall("tail".to_owned()).into()),
        }
    });
}

pub fn native_macros() -> HashMap<String, NativeMacro> {
    let mut macros: HashMap<String, NativeMacro> = HashMap::new();

    macros.insert("const".to_string(), |scope, args, line, col| {
        if args.len() != 2 {
            return Err(InterpreterError::InvalidMacroCall("const".to_owned()).into());
        }

        let name = match &args[0].ty {
            AstNodeType::Ident(t) if matches!(t, TokenIdent::Ident(..)) => t,
            _ => return Err(InterpreterError::InvalidMacroCall("const".to_owned()).into()),
        };

        let value = scope.evaluate(&args[1])?;

        scope.set_const(&name.without_generics(), value, line, col)?;

        Ok(Rc::new(InterpreterValue::Void))
    });

    // TODO: Hold optional variable type information
    macros.insert("let".to_string(), |scope, args, line, col| {
        if args.len() != 2 {
            return Err(InterpreterError::InvalidMacroCall("let".to_owned()).into());
        }

        let name = match &args[0].ty {
            AstNodeType::Ident(t) if matches!(t, TokenIdent::Ident(..)) => t,
            _ => return Err(InterpreterError::InvalidMacroCall("let".to_owned()).into()),
        };

        let value = scope.evaluate(&args[1])?;

        scope.set(&name.without_generics(), value, line, col)?;

        Ok(Rc::new(InterpreterValue::Void))
    });

    macros.insert("set".to_string(), |scope, args, line, col| {
        if args.len() != 2 {
            return Err(InterpreterError::InvalidMacroCall("set".to_owned()).into());
        }

        let name = match &args[0].ty {
            AstNodeType::Ident(t) if matches!(t, TokenIdent::Ident(..)) => t,
            _ => return Err(InterpreterError::InvalidMacroCall("set".to_owned()).into()),
        };

        let value = scope.evaluate(&args[1])?;

        scope.replace(&name.without_generics(), value, line, col)?;

        Ok(Rc::new(InterpreterValue::Void))
    });

    macros.insert("fn".to_string(), |scope, args, line, col| {
        if args.len() < 2 {
            return Err(InterpreterError::InvalidMacroCall("fn".to_owned()).into());
        }

        let name = match &args[0].ty {
            AstNodeType::Ident(t) if matches!(t, TokenIdent::Ident(..)) => t,
            _ => return Err(InterpreterError::InvalidMacroCall("fn".to_owned()).into()),
        };

        let mut params = Vec::new();

        let AstNodeType::Array(params_) = &args[1].ty else {
            return Err(InterpreterError::InvalidMacroCall("fn".to_owned()).into());
        };

        let mut iter = params_.into_iter().peekable();
        while let Some(param) = iter.next() {
            match &param.ty {
                AstNodeType::Ident(TokenIdent::Ident(s, None)) => params.push((
                    s.to_owned(),
                    match iter.peek() {
                        Some(AstNode {
                            ty: AstNodeType::Ident(t),
                            line,
                            col,
                        }) => {
                            if let TokenIdent::Type(..) = t {
                                match scope.get(t, *line, *col) {
                                    Ok(rc) => match rc.as_ref() {
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
                                    },
                                    Err(_) => {
                                        iter.next();
                                        InterpreterType::ToGet(t.clone())
                                    }
                                }
                            } else {
                                InterpreterType::Any
                            }
                        }
                        _ => InterpreterType::Any,
                    },
                )),
                _ => return Err(InterpreterError::InvalidMacroCall("fn".to_owned()).into()),
            }
        }

        let (return_type, has) = match &args[2].ty {
            AstNodeType::Ident(t) if matches!(t, TokenIdent::Type(..)) => (
                match scope.get(t, line, col) {
                    Ok(rc) => match rc.as_ref() {
                        InterpreterValue::Type(t) => {
                            iter.next();
                            t.clone()
                        }
                        _ => return Err(InterpreterError::InvalidMacroCall("fn".to_owned()).into()),
                    },
                    Err(_) => {
                        iter.next();
                        InterpreterType::ToGet(t.clone())
                    }
                },
                true,
            ),
            _ => (InterpreterType::Any, false),
        };

        let body = args[if has { 3 } else { 2 }..].to_vec();

        let func = Rc::new(InterpreterValue::Function {
            name: name.name().to_owned(),
            generics: name.get_generics().map(|v| {
                v.iter()
                    .map(|v| (v.ident.name().to_owned(), v.type_ident.to_owned()))
                    .collect()
            }),
            params,
            return_type,
            body,
        });

        if scope.top_scope {
            scope.set_const(&name.without_generics(), func.clone(), line, col)?;

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

        let body = if condition { &args[1] } else { &args[2] };

        scope.evaluate(body)
    });

    macros.insert("if".to_string(), |scope, args, line, col| {
        if args.len() < 2 {
            return Err(InterpreterError::InvalidMacroCall("if".to_owned()).into());
        }

        let condition = match &args[0].ty {
            AstNodeType::Ident(s) => scope.get(s, line, col)?,
            AstNodeType::Bool(b) => Rc::new(InterpreterValue::Bool(*b)),
            AstNodeType::Call { .. } => scope.evaluate(&args[0])?,
            _ => return Err(InterpreterError::InvalidMacroCall("if".to_owned()).into()),
        };

        let condition = match condition.as_ref() {
            InterpreterValue::Bool(b) => *b,
            _ => return Err(InterpreterError::InvalidMacroCall("if".to_owned()).into()),
        };

        if condition {
            scope.evaluate_block(&args[1..])
        } else {
            Ok(Rc::new(InterpreterValue::Void))
        }
    });

    macros.insert("while".to_string(), |scope, args, line, col| {
        if args.len() < 2 {
            return Err(InterpreterError::InvalidMacroCall("while".to_owned()).into());
        }

        let mut result = Rc::new(InterpreterValue::Void);
        while match (match &args[0].ty {
            AstNodeType::Ident(s) => scope.get(s, line, col)?,
            AstNodeType::Bool(b) => Rc::new(InterpreterValue::Bool(*b)),
            AstNodeType::Call { .. } => scope.evaluate(&args[0])?,
            _ => return Err(InterpreterError::InvalidMacroCall("while".to_owned()).into()),
        })
        .as_ref()
        {
            InterpreterValue::Bool(b) => *b,
            _ => return Err(InterpreterError::InvalidMacroCall("while".to_owned()).into()),
        } {
            result = scope.evaluate_block(&args[1..])?;
        }
        Ok(result)
    });

    // TODO: Add type validation
    // e.g.
    // when `(@struct $Point x: $int, y: $int)` is defined,
    // `(@dict[$Point] x: 1, y: 2)` should be valid,
    // but `(@dict[$Point] x: 1, y: "2")` should be invalid
    //
    // or with generics
    // `(@struct[$T] $Point x: $T, y: $T)`
    // `(@dict[$Point[$int]] x: 1, y: 2)` should be valid
    // `(@dict[$Point[$int]] x: 1, y: "2")` should be invalid
    macros.insert("dict".to_string(), |scope, args, _, _| {
        if args.len() % 2 != 0 {
            return Err(InterpreterError::InvalidMacroCall("dict".to_owned()).into());
        }

        let mut dict = HashMap::new();

        for i in (0..args.len()).step_by(2) {
            let s = match &args[i].ty {
                AstNodeType::Ident(TokenIdent::Ident(s, None)) => s,
                _ => return Err(InterpreterError::InvalidMacroCall("dict".to_owned()).into()),
            };

            let value = scope.evaluate(&args[i + 1])?;

            dict.insert(s.to_owned(), value);
        }

        Ok(Rc::new(InterpreterValue::Dict(RefCell::new(dict))))
    });

    // creates a struct *type*, not an instance. use `dict` to create an instance
    // TODO: Support generics. e.g. `(@struct[$T] $Point x: $T, y: $T)`
    macros.insert("struct".to_string(), |scope, args, line, col| {
        if args.len() % 2 == 0 {
            return Err(InterpreterError::InvalidMacroCall("struct".to_owned()).into());
        }

        let name = match &args[0].ty {
            AstNodeType::Ident(TokenIdent::Type(s, None)) => s,
            _ => return Err(InterpreterError::InvalidMacroCall("struct".to_owned()).into()),
        };

        let mut fields = Vec::new();

        for i in (1..args.len()).step_by(2) {
            let s = match &args[i].ty {
                AstNodeType::Ident(TokenIdent::Ident(s, None)) => s,
                _ => return Err(InterpreterError::InvalidMacroCall("struct".to_owned()).into()),
            };

            let value = match &args[i + 1].ty {
                AstNodeType::Ident(t) if matches!(t, TokenIdent::Type(..)) => {
                    match scope.get(t, line, col) {
                        Ok(rc) => match rc.as_ref() {
                            InterpreterValue::Type(t) => {
                                fields.push((s.to_owned(), t.clone()));
                                continue;
                            }
                            _ => {
                                return Err(InterpreterError::InvalidMacroCall(
                                    "struct".to_owned(),
                                )
                                .into())
                            }
                        },
                        Err(_) => InterpreterType::ToGet(t.clone()),
                    }
                }
                _ => return Err(InterpreterError::InvalidMacroCall("struct".to_owned()).into()),
            };

            fields.push((s.to_owned(), value));
        }

        let struct_type = Rc::new(InterpreterValue::Type(InterpreterType::Struct(
            fields,
        )));

        scope.set_const(&TokenIdent::Type(name.to_owned(), None), struct_type.clone(), line, col)?;

        Ok(struct_type)
    });

    macros
}
