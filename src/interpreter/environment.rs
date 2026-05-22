use crate::expressions::Value;
use std::collections::HashMap;

pub struct Environment<'a> {
    enclosing: Option<&'a Environment<'a>>,
    values: HashMap<&'a str, Option<Box<Value>>>,
}

impl<'a> Environment<'a> {
    pub fn new(enclosing: Option<&'a Environment<'a>>) -> Self {
        Environment {
            enclosing,
            values: HashMap::new(),
        }
    }

    pub fn define(&mut self, name: &'a str, value: Option<Box<Value>>) {
        self.values.insert(name, value);
    }

    pub fn get(&self, name: &str) -> Option<&Option<Box<Value>>> {
        return self.values.get(name);
    }
}

