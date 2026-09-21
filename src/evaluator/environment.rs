use crate::evaluator::globals::define_globals;
use crate::expressions::Value;
use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt::Debug;
use std::rc::Rc;

#[derive(Eq, Hash, PartialEq)]
pub struct VariableBinding<'a> {
    pub line: usize,
    pub name: &'a str,
}

pub type LookupMap<'a> = HashMap<VariableBinding<'a>, usize>;

#[derive(Clone)]
pub struct Frame<'a>(Rc<RefCell<HashMap<&'a str, Option<Value<'a>>>>>);

impl<'a> Frame<'a> {
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

impl Debug for Frame<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Frame").field(&self.0).finish()
    }
}

pub struct Environment<'a> {
    frames: Vec<Frame<'a>>,
    locals: LookupMap<'a>,
    globals: HashMap<&'a str, Option<Value<'a>>>,
}

pub enum GetError {
    Undefined,
    Uninitalised,
}

impl<'a> Environment<'a> {
    pub fn new(locals: LookupMap<'a>) -> Self {
        let mut env = Environment {
            locals,
            frames: vec![Frame::new()],
            globals: HashMap::new(),
        };
        define_globals(&mut env);
        env
    }

    pub fn narrow(&mut self) {
        self.frames.push(Frame::new());
    }

    pub fn push(&mut self, frame: Frame<'a>) {
        self.frames.push(frame);
    }

    pub fn pop(&mut self) -> Frame<'a> {
        self.frames
            .pop()
            .expect("One hashmap should be initalised at all times")
    }

    pub fn top(&self) -> &Frame<'a> {
        self.frames
            .last()
            .expect("One hashmap should be initalised at all times")
    }

    pub fn define(&mut self, name: &'a str, value: Option<Value<'a>>) {
        self.frames
            .last_mut()
            .expect("One hashmap should be initalised at all times")
            .insert(name, value);
    }

    pub fn update(
        &mut self,
        name: &'a str,
        value: Value<'a>,
    ) -> Result<(), ()> {
        for frame in self.frames.iter_mut().rev() {
            if frame.contains_key(name) {
                frame.insert(name, Some(value));
                return Ok(());
            }
        }
        Err(())
    }

    pub fn get(
        &self,
        binding: &VariableBinding<'a>,
    ) -> Result<Value<'a>, GetError> {
        let distance = self.locals.get(binding);
        distance.map_or_else(
            || {
                self.globals.get(binding.name).map_or(
                    Err(GetError::Undefined),
                    |value| {
                        value
                            .as_ref()
                            .map_or(Err(GetError::Uninitalised), |v| {
                                Ok(v.clone())
                            })
                    },
                )
            },
            |dist| self.frames[self.frames.len() - dist].get(binding.name),
        )
    }

    pub fn add_global(&mut self, name: &'a str, value: Value<'a>) {
        self.globals.insert(name, Some(value));
    }
}
