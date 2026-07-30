use crate::evaluator::globals::define_globals;
use crate::expressions::Value;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

pub type Scope<'a> = Rc<RefCell<HashMap<&'a str, Option<Value<'a>>>>>;

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
            scopes: vec![Rc::new(RefCell::new(HashMap::new()))],
        };
        define_globals(&mut env);
        env
    }

    pub fn narrow(&mut self) {
        self.scopes.push(Rc::new(RefCell::new(HashMap::new())));
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
            .borrow_mut()
            .insert(name, value);
    }

    pub fn update(
        &mut self,
        name: &'a str,
        value: Value<'a>,
    ) -> Result<(), ()> {
        for scope in self.scopes.iter_mut().rev() {
            if scope.borrow().contains_key(name) {
                scope.borrow_mut().insert(name, Some(value));
                return Ok(());
            }
        }
        Err(())
    }

    pub fn get(&self, name: &str) -> Result<Value<'a>, GetError> {
        for scope in self.scopes.iter().rev() {
            if let Some(value) = scope.borrow().get(name) {
                return value.clone().ok_or(GetError::Uninitalised);
            }
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
