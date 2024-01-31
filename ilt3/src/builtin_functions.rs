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

    define_builtin_function!(scope, "float_add", ["a", "b", "..."], |args| {
        let mut result = 0.0;
        for arg in args {
            let value = arg
                .as_float()
                .ok_or(anyhow!("Expected float. Got {:?}", arg))?;
            result += value;
        }
        Ok(Value::Float(result))
    });
}
