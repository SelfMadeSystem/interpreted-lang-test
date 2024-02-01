use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::value::Value;

#[derive(Debug, Clone)]
pub struct Scope {
    pub parent: Option<Rc<RefCell<Scope>>>,
    pub named_variables: HashMap<String, Rc<RefCell<Value>>>,
    pub local_variables: Vec<Rc<RefCell<Value>>>,
}

impl Scope {
    pub fn new() -> Self {
        Self {
            parent: None,
            named_variables: HashMap::new(),
            local_variables: Vec::new(),
        }
    }

    pub fn new_child(parent: Rc<RefCell<Scope>>) -> Self {
        Self {
            parent: Some(parent),
            named_variables: HashMap::new(),
            local_variables: Vec::new(),
        }
    }

    pub fn get_named(&self, name: &str) -> Option<Rc<RefCell<Value>>> {
        if let Some(value) = self.named_variables.get(name) {
            return Some(value.clone());
        }

        if let Some(parent) = &self.parent {
            return parent.borrow().get_named(name);
        }

        None
    }

    pub fn set_named(&mut self, name: String, value: Rc<RefCell<Value>>) {
        self.named_variables.insert(name, value);
    }

    pub fn get_local(&self, index: usize) -> Option<Rc<RefCell<Value>>> {
        if let Some(value) = self.local_variables.get(index) {
            return Some(value.clone());
        }

        None
    }

    pub fn set_local(&mut self, index: usize, value: Rc<RefCell<Value>>) {
        if index >= self.local_variables.len() {
            self.local_variables.resize(index + 1, Rc::new(RefCell::new(Value::Void)));
        }
        self.local_variables[index] = value;
    }
}
