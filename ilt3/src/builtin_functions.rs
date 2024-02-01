use std::{cell::RefCell, rc::Rc};

use anyhow::anyhow;

use crate::{
    scope::Scope,
    value::{Value, ValueFunction, ValueFunctionBody},
};

macro_rules! define_builtin_function {
    ($scope: expr, $name:expr, $args:expr, $body:expr) => {
        $scope.set_named(
            $name.to_owned(),
            Rc::new(RefCell::new(Value::Function(ValueFunction {
                args: $args,
                body: ValueFunctionBody::Native($body),
            }))),
        );
    };
}

pub fn add_builtin_functions(scope: &mut Scope) {
    define_builtin_function!(scope, "print", 0, |args| {
        for arg in args {
            print!("{:?}", arg.borrow());
        }
        println!();
        Ok(Rc::new(RefCell::new(Value::Void)))
    });

    define_builtin_function!(scope, "time", 0, |_| {
        let time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("Failed to get time")
            .as_secs_f64();
        Ok(Rc::new(RefCell::new(Value::Float(time))))
    });

    add_array_functions(scope);
    add_int_functions(scope);
    add_float_functions(scope);
}

pub fn add_array_functions(scope: &mut Scope) {
    define_builtin_function!(scope, "array_is_empty", 1, |args| {
        let array = args.get(0).ok_or(anyhow!("No arguments passed."))?;
        let array = array.borrow();
        let array = array
            .as_array()
            .ok_or(anyhow!("Expected array."))?;
        Ok(Rc::new(RefCell::new(Value::Bool(array.is_empty()))))
    });

    define_builtin_function!(scope, "array_len", 1, |args| {
        let array = args.get(0).ok_or(anyhow!("No arguments passed."))?;
        let array = array.borrow();
        let array = array
            .as_array()
            .ok_or(anyhow!("Expected array."))?;
        Ok(Rc::new(RefCell::new(Value::Int(array.len() as i64))))
    });

    define_builtin_function!(scope, "array_head", 1, |args| {
        let array = args.get(0).ok_or(anyhow!("No arguments passed."))?;
        let array = array.borrow();
        let array = array
            .as_array()
            .ok_or(anyhow!("Expected array."))?;
        Ok(array
            .first()
            .cloned()
            .unwrap_or_else(|| Rc::new(RefCell::new(Value::Void))))
    });

    define_builtin_function!(scope, "array_tail", 0, |args| {
        let array = args.get(0).ok_or(anyhow!("No arguments passed."))?;
        let array = array.borrow();
        let array = array
            .as_array()
            .ok_or(anyhow!("Expected array."))?;
        Ok(Rc::new(RefCell::new(Value::Array(array[1..].to_vec()))))
    });
}

pub fn add_int_functions(scope: &mut Scope) {
    define_builtin_function!(scope, "int_add", 2, |args| {
        let mut result = 0;
        for arg in args {
            let arg = arg.borrow().as_int().ok_or(anyhow!("Expected int."))?;
            result += arg;
        }
        Ok(Rc::new(RefCell::new(Value::Int(result))))
    });

    define_builtin_function!(scope, "int_sub", 2, |args| {
        let a = args.get(0).ok_or(anyhow!("No arguments passed."))?;
        let a = a.borrow().as_int().ok_or(anyhow!("Expected int."))?;
        let b = args.get(1).ok_or(anyhow!("No arguments passed."))?;
        let b = b.borrow().as_int().ok_or(anyhow!("Expected int."))?;
        Ok(Rc::new(RefCell::new(Value::Int(a - b))))
    });

    define_builtin_function!(scope, "int_mul", 2, |args| {
        let mut result = 1;
        for arg in args {
            let arg = arg.borrow().as_int().ok_or(anyhow!("Expected int."))?;
            result *= arg;
        }
        Ok(Rc::new(RefCell::new(Value::Int(result))))
    });

    define_builtin_function!(scope, "int_div", 2, |args| {
        let a = args.get(0).ok_or(anyhow!("No arguments passed."))?;
        let a = a.borrow().as_int().ok_or(anyhow!("Expected int."))?;
        let b = args.get(1).ok_or(anyhow!("No arguments passed."))?;
        let b = b.borrow().as_int().ok_or(anyhow!("Expected int."))?;
        Ok(Rc::new(RefCell::new(Value::Int(a / b))))
    });

    define_builtin_function!(scope, "int_mod", 2, |args| {
        let a = args.get(0).ok_or(anyhow!("No arguments passed."))?;
        let a = a.borrow().as_int().ok_or(anyhow!("Expected int."))?;
        let b = args.get(1).ok_or(anyhow!("No arguments passed."))?;
        let b = b.borrow().as_int().ok_or(anyhow!("Expected int."))?;
        Ok(Rc::new(RefCell::new(Value::Int(a % b))))
    });

    define_builtin_function!(scope, "int_eq", 2, |args| {
        let a = args.get(0).ok_or(anyhow!("No arguments passed."))?;
        let a = a.borrow().as_int().ok_or(anyhow!("Expected int."))?;
        let b = args.get(1).ok_or(anyhow!("No arguments passed."))?;
        let b = b.borrow().as_int().ok_or(anyhow!("Expected int."))?;
        Ok(Rc::new(RefCell::new(Value::Bool(a == b))))
    });

    define_builtin_function!(scope, "int_neq", 2, |args| {
        let a = args.get(0).ok_or(anyhow!("No arguments passed."))?;
        let a = a.borrow().as_int().ok_or(anyhow!("Expected int."))?;
        let b = args.get(1).ok_or(anyhow!("No arguments passed."))?;
        let b = b.borrow().as_int().ok_or(anyhow!("Expected int."))?;
        Ok(Rc::new(RefCell::new(Value::Bool(a != b))))
    });

    define_builtin_function!(scope, "int_lt", 2, |args| {
        let a = args.get(0).ok_or(anyhow!("No arguments passed."))?;
        let a = a.borrow().as_int().ok_or(anyhow!("Expected int."))?;
        let b = args.get(1).ok_or(anyhow!("No arguments passed."))?;
        let b = b.borrow().as_int().ok_or(anyhow!("Expected int."))?;
        Ok(Rc::new(RefCell::new(Value::Bool(a < b))))
    });

    define_builtin_function!(scope, "int_le", 2, |args| {
        let a = args.get(0).ok_or(anyhow!("No arguments passed."))?;
        let a = a.borrow().as_int().ok_or(anyhow!("Expected int."))?;
        let b = args.get(1).ok_or(anyhow!("No arguments passed."))?;
        let b = b.borrow().as_int().ok_or(anyhow!("Expected int."))?;
        Ok(Rc::new(RefCell::new(Value::Bool(a <= b))))
    });

    define_builtin_function!(scope, "int_gt", 2, |args| {
        let a = args.get(0).ok_or(anyhow!("No arguments passed."))?;
        let a = a.borrow().as_int().ok_or(anyhow!("Expected int."))?;
        let b = args.get(1).ok_or(anyhow!("No arguments passed."))?;
        let b = b.borrow().as_int().ok_or(anyhow!("Expected int."))?;
        Ok(Rc::new(RefCell::new(Value::Bool(a > b))))
    });

    define_builtin_function!(scope, "int_ge", 2, |args| {
        let a = args.get(0).ok_or(anyhow!("No arguments passed."))?;
        let a = a.borrow().as_int().ok_or(anyhow!("Expected int."))?;
        let b = args.get(1).ok_or(anyhow!("No arguments passed."))?;
        let b = b.borrow().as_int().ok_or(anyhow!("Expected int."))?;
        Ok(Rc::new(RefCell::new(Value::Bool(a >= b))))
    });

    define_builtin_function!(scope, "int_to_float", 1, |args| {
        let int = args.get(0).ok_or(anyhow!("No arguments passed."))?;
        let int = int.borrow().as_int().ok_or(anyhow!("Expected int."))?;
        Ok(Rc::new(RefCell::new(Value::Float(int as f64))))
    });

    define_builtin_function!(scope, "int_to_string", 1, |args| {
        let int = args.get(0).ok_or(anyhow!("No arguments passed."))?;
        let int = int.borrow().as_int().ok_or(anyhow!("Expected int."))?;
        Ok(Rc::new(RefCell::new(Value::String(int.to_string()))))
    });

    define_builtin_function!(scope, "int_to_bool", 1, |args| {
        let int = args.get(0).ok_or(anyhow!("No arguments passed."))?;
        let int = int.borrow().as_int().ok_or(anyhow!("Expected int."))?;
        Ok(Rc::new(RefCell::new(Value::Bool(int != 0))))
    });
}

pub fn add_float_functions(scope: &mut Scope) {
    define_builtin_function!(scope, "float_add", 2, |args| {
        let mut result = 0.;
        for arg in args {
            let arg = arg.borrow().as_float().ok_or(anyhow!("Expected float."))?;
            result += arg;
        }
        Ok(Rc::new(RefCell::new(Value::Float(result))))
    });

    define_builtin_function!(scope, "float_sub", 2, |args| {
        let a = args.get(0).ok_or(anyhow!("No arguments passed."))?;
        let a = a.borrow().as_float().ok_or(anyhow!("Expected float."))?;
        let b = args.get(1).ok_or(anyhow!("No arguments passed."))?;
        let b = b.borrow().as_float().ok_or(anyhow!("Expected float."))?;
        Ok(Rc::new(RefCell::new(Value::Float(a - b))))
    });

    define_builtin_function!(scope, "float_mul", 2, |args| {
        let mut result = 1.;
        for arg in args {
            let arg = arg.borrow().as_float().ok_or(anyhow!("Expected float."))?;
            result *= arg;
        }
        Ok(Rc::new(RefCell::new(Value::Float(result))))
    });

    define_builtin_function!(scope, "float_div", 2, |args| {
        let a = args.get(0).ok_or(anyhow!("No arguments passed."))?;
        let a = a.borrow().as_float().ok_or(anyhow!("Expected float."))?;
        let b = args.get(1).ok_or(anyhow!("No arguments passed."))?;
        let b = b.borrow().as_float().ok_or(anyhow!("Expected float."))?;
        Ok(Rc::new(RefCell::new(Value::Float(a / b))))
    });

    define_builtin_function!(scope, "float_mod", 2, |args| {
        let a = args.get(0).ok_or(anyhow!("No arguments passed."))?;
        let a = a.borrow().as_float().ok_or(anyhow!("Expected float."))?;
        let b = args.get(1).ok_or(anyhow!("No arguments passed."))?;
        let b = b.borrow().as_float().ok_or(anyhow!("Expected float."))?;
        Ok(Rc::new(RefCell::new(Value::Float(a % b))))
    });

    define_builtin_function!(scope, "float_eq", 2, |args| {
        let a = args.get(0).ok_or(anyhow!("No arguments passed."))?;
        let a = a.borrow().as_float().ok_or(anyhow!("Expected float."))?;
        let b = args.get(1).ok_or(anyhow!("No arguments passed."))?;
        let b = b.borrow().as_float().ok_or(anyhow!("Expected float."))?;
        Ok(Rc::new(RefCell::new(Value::Bool(a == b))))
    });

    define_builtin_function!(scope, "float_neq", 2, |args| {
        let a = args.get(0).ok_or(anyhow!("No arguments passed."))?;
        let a = a.borrow().as_float().ok_or(anyhow!("Expected float."))?;
        let b = args.get(1).ok_or(anyhow!("No arguments passed."))?;
        let b = b.borrow().as_float().ok_or(anyhow!("Expected float."))?;
        Ok(Rc::new(RefCell::new(Value::Bool(a != b))))
    });

    define_builtin_function!(scope, "float_lt", 2, |args| {
        let a = args.get(0).ok_or(anyhow!("No arguments passed."))?;
        let a = a.borrow().as_float().ok_or(anyhow!("Expected float."))?;
        let b = args.get(1).ok_or(anyhow!("No arguments passed."))?;
        let b = b.borrow().as_float().ok_or(anyhow!("Expected float."))?;
        Ok(Rc::new(RefCell::new(Value::Bool(a < b))))
    });

    define_builtin_function!(scope, "float_le", 2, |args| {
        let a = args.get(0).ok_or(anyhow!("No arguments passed."))?;
        let a = a.borrow().as_float().ok_or(anyhow!("Expected float."))?;
        let b = args.get(1).ok_or(anyhow!("No arguments passed."))?;
        let b = b.borrow().as_float().ok_or(anyhow!("Expected float."))?;
        Ok(Rc::new(RefCell::new(Value::Bool(a <= b))))
    });

    define_builtin_function!(scope, "float_gt", 2, |args| {
        let a = args.get(0).ok_or(anyhow!("No arguments passed."))?;
        let a = a.borrow().as_float().ok_or(anyhow!("Expected float."))?;
        let b = args.get(1).ok_or(anyhow!("No arguments passed."))?;
        let b = b.borrow().as_float().ok_or(anyhow!("Expected float."))?;
        Ok(Rc::new(RefCell::new(Value::Bool(a > b))))
    });

    define_builtin_function!(scope, "float_ge", 2, |args| {
        let a = args.get(0).ok_or(anyhow!("No arguments passed."))?;
        let a = a.borrow().as_float().ok_or(anyhow!("Expected float."))?;
        let b = args.get(1).ok_or(anyhow!("No arguments passed."))?;
        let b = b.borrow().as_float().ok_or(anyhow!("Expected float."))?;
        Ok(Rc::new(RefCell::new(Value::Bool(a >= b))))
    });

    define_builtin_function!(scope, "float_to_int", 1, |args| {
        let float = args.get(0).ok_or(anyhow!("No arguments passed."))?;
        let float = float
            .borrow()
            .as_float()
            .ok_or(anyhow!("Expected float."))?;
        Ok(Rc::new(RefCell::new(Value::Int(float as i64))))
    });

    define_builtin_function!(scope, "float_to_string", 1, |args| {
        let float = args.get(0).ok_or(anyhow!("No arguments passed."))?;
        let float = float
            .borrow()
            .as_float()
            .ok_or(anyhow!("Expected float."))?;
        Ok(Rc::new(RefCell::new(Value::String(float.to_string()))))
    });

    define_builtin_function!(scope, "float_to_bool", 1, |args| {
        let float = args.get(0).ok_or(anyhow!("No arguments passed."))?;
        let float = float
            .borrow()
            .as_float()
            .ok_or(anyhow!("Expected float."))?;
        Ok(Rc::new(RefCell::new(Value::Bool(float != 0.))))
    });
}
