use crate::expressions::Value;
use std::collections::HashMap;

pub struct Environment<'a> {
    enclosing: Option<&'a Self>,
    values: HashMap<&'a str, Value>,
}

impl<'a> Environment<'a> {
    pub fn new(enclosing: Option<&'a Self>) -> Self {
        Environment {
            enclosing,
            values: HashMap::new(),
        }
    }

    pub fn define(&mut self, name: &'a str, value: Option<Value>) {
        self.values.insert(name, value.unwrap_or(Value::Nil));
    }

    pub fn get(&self, name: &str) -> Option<&Value> {
        self.values.get(name)
    }
}
