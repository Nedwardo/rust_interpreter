use log::trace;

use crate::evaluator::globals::define_globals;
use crate::expressions::Value;
use crate::token::Span;
use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt;
use std::fmt::Debug;
use std::fmt::Display;
use std::fmt::Formatter;
use std::rc::Rc;

#[derive(Debug, Clone, Eq, Hash, PartialEq)]
pub struct VariableBinding<'a> {
    pub span: Span,
    pub name: &'a str,
}

impl<'a> VariableBinding<'a> {
    pub fn from_span(span: Span, source: &'a str) -> Self {
        Self {
            span,
            name: &source[span.0..span.1],
        }
    }
}

pub type LookupMap<'a> = HashMap<VariableBinding<'a>, usize>;

#[derive(Clone)]
pub struct Frame<'a>(Rc<RefCell<HashMap<&'a str, Option<Value<'a>>>>>);

impl<'a> Frame<'a> {
    pub fn new() -> Self {
        Self(Rc::new(RefCell::new(HashMap::new())))
    }

    pub fn insert(&self, key: &'a str, value: Option<Value<'a>>) {
        self.0.borrow_mut().insert(key, value);
    }

    pub fn contains_key(&self, key: &'a str) -> bool {
        self.0.borrow().contains_key(key)
    }

    pub fn get(&self, key: &'a str) -> Result<Value<'a>, GetError> {
        self.0
            .borrow()
            .get(key)
            .map_or(Err(GetError::Undefined), |value| {
                value.clone().ok_or(GetError::Uninitalised)
            })
    }
}

impl Display for Frame<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "Frame{{")?;
        for (i, (name, value)) in self.0.borrow().iter().enumerate() {
            if i > 0 {
                write!(f, ", ")?;
            }
            write!(f, "{name}: ")?;
            match value {
                None => write!(f, "Uninitalised"),
                Some(value) => write!(f, "{value}"),
            }?;
        }
        write!(f, "}}")
    }
}

impl Debug for Frame<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_tuple("Frame").field(&self.0).finish()
    }
}

pub struct Environment<'a> {
    frames: Vec<Frame<'a>>,
    locals: LookupMap<'a>,
    globals: Frame<'a>,
}

#[derive(Debug)]
pub enum GetError {
    Undefined,
    Uninitalised,
}

impl<'a> Environment<'a> {
    pub fn new(locals: LookupMap<'a>) -> Self {
        let env = Environment {
            locals,
            frames: Vec::new(),
            globals: Frame::new(),
        };
        define_globals(&env);
        env
    }

    pub fn narrow(&mut self) {
        trace!("Narrowing");
        self.frames.push(Frame::new());
    }

    pub fn push(&mut self, frame: Frame<'a>) {
        trace!("Entering func");
        self.frames.push(frame);
    }

    pub fn pop(&mut self) -> Option<Frame<'a>> {
        trace!("Popping");
        self.frames.pop()
    }

    pub fn top(&self) -> &Frame<'a> {
        self.frames.last().map_or(&self.globals, |frame| frame)
    }

    pub fn define(
        &mut self,
        binding: &VariableBinding<'a>,
        value: Option<Value<'a>>,
    ) {
        if let Some(frame) = self.frames.last_mut() {
            frame.insert(binding.name, value);
        } else {
            self.globals.insert(binding.name, value);
        }
    }

    pub fn update(
        &self,
        binding: &VariableBinding<'a>,
        value: Value<'a>,
    ) -> Result<(), ()> {
        let frame_size = self.frames.len();
        let distance = self.locals.get(binding);

        if let Some(dist) = distance {
            self.frames[frame_size - dist - 1]
                .insert(binding.name, Some(value));
            Ok(())
        } else if self.globals.contains_key(binding.name) {
            self.globals.insert(binding.name, Some(value));
            Ok(())
        } else {
            Err(())
        }
    }

    pub fn get(
        &self,
        binding: &VariableBinding<'a>,
    ) -> Result<Value<'a>, GetError> {
        let distance = self.locals.get(binding);
        distance.map_or_else(
            || self.globals.get(binding.name),
            |dist| self.frames[self.frames.len() - dist - 1].get(binding.name),
        )
    }

    pub fn get_local(&self, name: &'a str) -> Result<Value<'a>, GetError> {
        self.frames[self.frames.len() - 1].get(name)
    }

    pub fn add_global(&self, name: &'a str, value: Value<'a>) {
        self.globals.insert(name, Some(value));
    }
}

impl Display for Environment<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        writeln!(f, "Env{{")?;
        writeln!(f, "Globals: {},\nFrames: [", self.globals)?;

        for frame in &self.frames {
            writeln!(f, "\t{frame},")?;
        }
        write!(f, "]}}")
    }
}
