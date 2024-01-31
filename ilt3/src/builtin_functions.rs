use anyhow::anyhow;

use crate::{
    scope::Scope,
    value::{Value, ValueFunction, ValueFunctionBody},
};

macro_rules! define_builtin_function {
    ($scope: expr, $name:expr, [$($arg:expr),*], $body:expr) => {
        $scope.set(
            $name.to_owned(),
            Value::Function(ValueFunction {
                args: vec![$($arg.to_string()),*],
                body: ValueFunctionBody::Native($body),
            }),
        );
    };
}

pub fn add_builtin_functions(scope: &mut Scope) {
    define_builtin_function!(scope, "print", ["..."], |args| {
        for arg in args {
            print!("{:?}", arg);
        }
        println!();
        Ok(Value::Void)
    });

    define_builtin_function!(scope, "time", [], |_| {
        let time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("Failed to get time")
            .as_secs_f64();
        Ok(Value::Float(time))
    });

    add_array_functions(scope);
    add_int_functions(scope);
    add_float_functions(scope);
}

pub fn add_array_functions(scope: &mut Scope) {
    define_builtin_function!(scope, "array_is_empty", ["array"], |args| {
        let array = args.get(0).ok_or(anyhow!("No arguments passed."))?;
        let array = array.as_array().ok_or(anyhow!("Expected array."))?;
        Ok(Value::Bool(array.is_empty()))
    });

    define_builtin_function!(scope, "array_len", ["array"], |args| {
        let array = args.get(0).ok_or(anyhow!("No arguments passed."))?;
        let array = array.as_array().ok_or(anyhow!("Expected array."))?;
        Ok(Value::Int(array.len() as i64))
    });

    define_builtin_function!(scope, "array_head", ["array"], |args| {
        let array = args.get(0).ok_or(anyhow!("No arguments passed."))?;
        let array = array.as_array().ok_or(anyhow!("Expected array."))?;
        Ok(array.get(0).cloned().unwrap_or(Value::Void))
    });

    define_builtin_function!(scope, "array_tail", ["array"], |args| {
        let array = args.get(0).ok_or(anyhow!("No arguments passed."))?;
        let array = array.as_array().ok_or(anyhow!("Expected array."))?;
        Ok(Value::Array(array[1..].to_vec()))
    });
}

pub fn add_int_functions(scope: &mut Scope) {
    define_builtin_function!(scope, "int_add", ["a", "b", "..."], |args| {
        let mut result = 0;
        for arg in args {
            let arg = arg.as_int().ok_or(anyhow!("Expected int."))?;
            result += arg;
        }
        Ok(Value::Int(result))
    });

    define_builtin_function!(scope, "int_sub", ["a", "b"], |args| {
        let a = args.get(0).ok_or(anyhow!("No arguments passed."))?;
        let a = a.as_int().ok_or(anyhow!("Expected int."))?;
        let b = args.get(1).ok_or(anyhow!("No arguments passed."))?;
        let b = b.as_int().ok_or(anyhow!("Expected int."))?;
        Ok(Value::Int(a - b))
    });

    define_builtin_function!(scope, "int_mul", ["a", "b", "..."], |args| {
        let mut result = 1;
        for arg in args {
            let arg = arg.as_int().ok_or(anyhow!("Expected int."))?;
            result *= arg;
        }
        Ok(Value::Int(result))
    });

    define_builtin_function!(scope, "int_div", ["a", "b"], |args| {
        let a = args.get(0).ok_or(anyhow!("No arguments passed."))?;
        let a = a.as_int().ok_or(anyhow!("Expected int."))?;
        let b = args.get(1).ok_or(anyhow!("No arguments passed."))?;
        let b = b.as_int().ok_or(anyhow!("Expected int."))?;
        Ok(Value::Int(a / b))
    });

    define_builtin_function!(scope, "int_mod", ["a", "b"], |args| {
        let a = args.get(0).ok_or(anyhow!("No arguments passed."))?;
        let a = a.as_int().ok_or(anyhow!("Expected int."))?;
        let b = args.get(1).ok_or(anyhow!("No arguments passed."))?;
        let b = b.as_int().ok_or(anyhow!("Expected int."))?;
        Ok(Value::Int(a % b))
    });

    define_builtin_function!(scope, "int_eq", ["a", "b"], |args| {
        let a = args.get(0).ok_or(anyhow!("No arguments passed."))?;
        let a = a.as_int().ok_or(anyhow!("Expected int."))?;
        let b = args.get(1).ok_or(anyhow!("No arguments passed."))?;
        let b = b.as_int().ok_or(anyhow!("Expected int."))?;
        Ok(Value::Bool(a == b))
    });

    define_builtin_function!(scope, "int_neq", ["a", "b"], |args| {
        let a = args.get(0).ok_or(anyhow!("No arguments passed."))?;
        let a = a.as_int().ok_or(anyhow!("Expected int."))?;
        let b = args.get(1).ok_or(anyhow!("No arguments passed."))?;
        let b = b.as_int().ok_or(anyhow!("Expected int."))?;
        Ok(Value::Bool(a != b))
    });

    define_builtin_function!(scope, "int_lt", ["a", "b"], |args| {
        let a = args.get(0).ok_or(anyhow!("No arguments passed."))?;
        let a = a.as_int().ok_or(anyhow!("Expected int."))?;
        let b = args.get(1).ok_or(anyhow!("No arguments passed."))?;
        let b = b.as_int().ok_or(anyhow!("Expected int."))?;
        Ok(Value::Bool(a < b))
    });

    define_builtin_function!(scope, "int_le", ["a", "b"], |args| {
        let a = args.get(0).ok_or(anyhow!("No arguments passed."))?;
        let a = a.as_int().ok_or(anyhow!("Expected int."))?;
        let b = args.get(1).ok_or(anyhow!("No arguments passed."))?;
        let b = b.as_int().ok_or(anyhow!("Expected int."))?;
        Ok(Value::Bool(a <= b))
    });

    define_builtin_function!(scope, "int_gt", ["a", "b"], |args| {
        let a = args.get(0).ok_or(anyhow!("No arguments passed."))?;
        let a = a.as_int().ok_or(anyhow!("Expected int."))?;
        let b = args.get(1).ok_or(anyhow!("No arguments passed."))?;
        let b = b.as_int().ok_or(anyhow!("Expected int."))?;
        Ok(Value::Bool(a > b))
    });

    define_builtin_function!(scope, "int_ge", ["a", "b"], |args| {
        let a = args.get(0).ok_or(anyhow!("No arguments passed."))?;
        let a = a.as_int().ok_or(anyhow!("Expected int."))?;
        let b = args.get(1).ok_or(anyhow!("No arguments passed."))?;
        let b = b.as_int().ok_or(anyhow!("Expected int."))?;
        Ok(Value::Bool(a >= b))
    });

    define_builtin_function!(scope, "int_to_float", ["int"], |args| {
        let int = args.get(0).ok_or(anyhow!("No arguments passed."))?;
        let int = int.as_int().ok_or(anyhow!("Expected int."))?;
        Ok(Value::Float(int as f64))
    });

    define_builtin_function!(scope, "int_to_string", ["int"], |args| {
        let int = args.get(0).ok_or(anyhow!("No arguments passed."))?;
        let int = int.as_int().ok_or(anyhow!("Expected int."))?;
        Ok(Value::String(int.to_string()))
    });

    define_builtin_function!(scope, "int_to_bool", ["int"], |args| {
        let int = args.get(0).ok_or(anyhow!("No arguments passed."))?;
        let int = int.as_int().ok_or(anyhow!("Expected int."))?;
        Ok(Value::Bool(int != 0))
    });
}

pub fn add_float_functions(scope: &mut Scope) {
    define_builtin_function!(scope, "float_add", ["a", "b", "..."], |args| {
        let mut result = 0.;
        for arg in args {
            let arg = arg.as_float().ok_or(anyhow!("Expected float."))?;
            result += arg;
        }
        Ok(Value::Float(result))
    });

    define_builtin_function!(scope, "float_sub", ["a", "b"], |args| {
        let a = args.get(0).ok_or(anyhow!("No arguments passed."))?;
        let a = a.as_float().ok_or(anyhow!("Expected float."))?;
        let b = args.get(1).ok_or(anyhow!("No arguments passed."))?;
        let b = b.as_float().ok_or(anyhow!("Expected float."))?;
        Ok(Value::Float(a - b))
    });

    define_builtin_function!(scope, "float_mul", ["a", "b", "..."], |args| {
        let mut result = 1.;
        for arg in args {
            let arg = arg.as_float().ok_or(anyhow!("Expected float."))?;
            result *= arg;
        }
        Ok(Value::Float(result))
    });

    define_builtin_function!(scope, "float_div", ["a", "b"], |args| {
        let a = args.get(0).ok_or(anyhow!("No arguments passed."))?;
        let a = a.as_float().ok_or(anyhow!("Expected float."))?;
        let b = args.get(1).ok_or(anyhow!("No arguments passed."))?;
        let b = b.as_float().ok_or(anyhow!("Expected float."))?;
        Ok(Value::Float(a / b))
    });

    define_builtin_function!(scope, "float_mod", ["a", "b"], |args| {
        let a = args.get(0).ok_or(anyhow!("No arguments passed."))?;
        let a = a.as_float().ok_or(anyhow!("Expected float."))?;
        let b = args.get(1).ok_or(anyhow!("No arguments passed."))?;
        let b = b.as_float().ok_or(anyhow!("Expected float."))?;
        Ok(Value::Float(a % b))
    });

    define_builtin_function!(scope, "float_eq", ["a", "b"], |args| {
        let a = args.get(0).ok_or(anyhow!("No arguments passed."))?;
        let a = a.as_float().ok_or(anyhow!("Expected float."))?;
        let b = args.get(1).ok_or(anyhow!("No arguments passed."))?;
        let b = b.as_float().ok_or(anyhow!("Expected float."))?;
        Ok(Value::Bool(a == b))
    });

    define_builtin_function!(scope, "float_neq", ["a", "b"], |args| {
        let a = args.get(0).ok_or(anyhow!("No arguments passed."))?;
        let a = a.as_float().ok_or(anyhow!("Expected float."))?;
        let b = args.get(1).ok_or(anyhow!("No arguments passed."))?;
        let b = b.as_float().ok_or(anyhow!("Expected float."))?;
        Ok(Value::Bool(a != b))
    });

    define_builtin_function!(scope, "float_lt", ["a", "b"], |args| {
        let a = args.get(0).ok_or(anyhow!("No arguments passed."))?;
        let a = a.as_float().ok_or(anyhow!("Expected float."))?;
        let b = args.get(1).ok_or(anyhow!("No arguments passed."))?;
        let b = b.as_float().ok_or(anyhow!("Expected float."))?;
        Ok(Value::Bool(a < b))
    });

    define_builtin_function!(scope, "float_le", ["a", "b"], |args| {
        let a = args.get(0).ok_or(anyhow!("No arguments passed."))?;
        let a = a.as_float().ok_or(anyhow!("Expected float."))?;
        let b = args.get(1).ok_or(anyhow!("No arguments passed."))?;
        let b = b.as_float().ok_or(anyhow!("Expected float."))?;
        Ok(Value::Bool(a <= b))
    });

    define_builtin_function!(scope, "float_gt", ["a", "b"], |args| {
        let a = args.get(0).ok_or(anyhow!("No arguments passed."))?;
        let a = a.as_float().ok_or(anyhow!("Expected float."))?;
        let b = args.get(1).ok_or(anyhow!("No arguments passed."))?;
        let b = b.as_float().ok_or(anyhow!("Expected float."))?;
        Ok(Value::Bool(a > b))
    });

    define_builtin_function!(scope, "float_ge", ["a", "b"], |args| {
        let a = args.get(0).ok_or(anyhow!("No arguments passed."))?;
        let a = a.as_float().ok_or(anyhow!("Expected float."))?;
        let b = args.get(1).ok_or(anyhow!("No arguments passed."))?;
        let b = b.as_float().ok_or(anyhow!("Expected float."))?;
        Ok(Value::Bool(a >= b))
    });

    define_builtin_function!(scope, "float_to_int", ["float"], |args| {
        let float = args.get(0).ok_or(anyhow!("No arguments passed."))?;
        let float = float.as_float().ok_or(anyhow!("Expected float."))?;
        Ok(Value::Int(float as i64))
    });

    define_builtin_function!(scope, "float_to_string", ["float"], |args| {
        let float = args.get(0).ok_or(anyhow!("No arguments passed."))?;
        let float = float.as_float().ok_or(anyhow!("Expected float."))?;
        Ok(Value::String(float.to_string()))
    });

    define_builtin_function!(scope, "float_to_bool", ["float"], |args| {
        let float = args.get(0).ok_or(anyhow!("No arguments passed."))?;
        let float = float.as_float().ok_or(anyhow!("Expected float."))?;
        Ok(Value::Bool(float != 0.0))
    });
}
