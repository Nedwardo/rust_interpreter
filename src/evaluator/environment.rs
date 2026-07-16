use crate::evaluator::globals::define_globals;
use crate::expressions::Value;
use core::fmt;
use std::collections::HashMap;
use std::fmt::{Display, Formatter};

#[derive(Debug)]
pub struct Environment<'a> {
    scopes: Vec<HashMap<&'a str, Option<Value<'a>>>>,
}

impl<'a> Environment<'a> {
    pub fn new() -> Self {
        let mut env = Environment {
            scopes: vec![HashMap::new()],
        };
        define_globals(&mut env);
        env
    }

    pub fn narrow(&mut self) {
        self.scopes.push(HashMap::new());
    }

    pub fn pop_scope(&mut self) {
        self.scopes.pop();
    }

    pub fn define(&mut self, name: &'a str, value: Option<Value<'a>>) {
        self.scopes
            .last_mut()
            .expect("One hashmap should be initalised at all times")
            .insert(name, value);
    }

    pub fn update(
        &mut self,
        name: &'a str,
        value: Value<'a>,
    ) -> Result<(), ()> {
        for scope in self.scopes.iter_mut().rev() {
            if scope.contains_key(name) {
                scope.insert(name, Some(value));
                return Ok(());
            }
        }
        Err(())
    }

    pub fn get(&self, name: &str) -> Option<&Option<Value<'a>>> {
        for scope in self.scopes.iter().rev() {
            if scope.contains_key(name) {
                return scope.get(name);
            }
        }
        None
    }

    pub fn add_global(&mut self, name: &'a str, value: Value<'a>) {
        self.scopes
            .first_mut()
            .expect("One hashmap should initalised at all times")
            .insert(name, Some(value));
    }
}
