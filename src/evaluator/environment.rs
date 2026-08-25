use crate::evaluator::globals::define_globals;
use crate::expressions::Value;
use std::borrow::BorrowMut;
use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt::Debug;
use std::rc::Rc;

#[derive(Clone)]
pub struct Scope<'a>(Rc<RefCell<HashMap<&'a str, Option<Value<'a>>>>>);

impl<'a> Scope<'a> {
    pub fn new() -> Self {
        Self(Rc::new(RefCell::new(HashMap::new())))
    }

    pub fn insert(&self, key: &'a str, value: Option<Value<'a>>) {
        (*self.0).borrow_mut().insert(key, value);
    }

    pub fn contains_key(&self, key: &'a str) -> bool {
        self.0.borrow().contains_key(key)
    }

    pub fn get(&self, key: &'a str) -> Result<Value<'a>, GetError> {
        let binding = self.0.borrow();
        binding.get(key).map_or(Err(GetError::Undefined), |value| {
            value
                .as_ref()
                .map_or(Err(GetError::Uninitalised), |v| Ok(v.clone()))
        })
        // todo!("Fix this to not clone, idk how to atm");
    }
}

impl Debug for Scope<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Scope").field(&self.0).finish()
    }
}

pub struct Environment<'a> {
    scopes: Vec<Scope<'a>>,
}

pub enum GetError {
    Undefined,
    Uninitalised,
}

impl<'a> Environment<'a> {
    pub fn new() -> Self {
        let mut env = Environment {
            scopes: vec![Scope::new()],
        };
        define_globals(&mut env);
        env
    }

    pub fn narrow(&mut self) {
        self.scopes.push(Scope::new());
    }

    pub fn push(&mut self, scope: Scope<'a>) {
        self.scopes.push(scope);
    }

    pub fn pop(&mut self) -> Scope<'a> {
        self.scopes
            .pop()
            .expect("One hashmap should be initalised at all times")
    }

    pub fn top(&self) -> &Scope<'a> {
        self.scopes
            .last()
            .expect("One hashmap should be initalised at all times")
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

    pub fn get(&self, name: &'a str) -> Result<Value<'a>, GetError> {
        for scope in self.scopes.iter().rev() {
            let value = scope.get(name);
            if matches!(value, Err(GetError::Undefined)) {
                continue;
            }

            return value;
        }
        Err(GetError::Undefined)
    }

    pub fn add_global(&mut self, name: &'a str, value: Value<'a>) {
        self.scopes
            .first_mut()
            .expect("One hashmap should initalised at all times")
            .borrow_mut()
            .insert(name, Some(value));
    }
}
