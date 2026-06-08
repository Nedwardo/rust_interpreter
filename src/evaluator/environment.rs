use crate::expressions::Value;
use std::collections::HashMap;

pub struct Environment<'a> {
    scopes: Vec<HashMap<&'a str, Value>>,
}

impl<'a> Environment<'a> {
    pub fn new() -> Self {
        Environment {
            scopes: vec![HashMap::new()],
        }
    }

    pub fn narrow(&mut self) {
        self.scopes.push(HashMap::new());
    }

    pub fn pop_scope(&mut self) {
        self.scopes.pop();
    }

    pub fn define(&mut self, name: &'a str, value: Option<Value>) {
        self.scopes
            .last_mut()
            .expect("One hashmap should be initalised at all times")
            .insert(name, value.unwrap_or(Value::Nil));
    }

    pub fn update(&mut self, name: &'a str, value: Value) -> Result<(), ()> {
        for scope in self.scopes.iter_mut().rev() {
            if scope.contains_key(name) {
                scope.insert(name, value);
                return Ok(());
            }
        }
        Err(())
    }

    pub fn get(&self, name: &str) -> Option<&Value> {
        for scope in self.scopes.iter().rev() {
            if scope.contains_key(name) {
                return scope.get(name);
            }
        }
        None
    }
}
