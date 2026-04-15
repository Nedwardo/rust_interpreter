use std::error;
use std::fmt::{Display, Formatter, Result};

#[derive(Debug)]
pub struct InterpreterError<'a> {
    pub line: usize,
    pub message: &'static str,
    pub error_location: Option<&'a str>,
}

impl<'a> Display for InterpreterError<'a> {
    fn fmt(&self, formatter: &mut Formatter) -> Result {
        match self.error_location {
            Some(error_location) => {
                write!(
                    formatter,
                    "[line {}] io::Error {}: {}",
                    self.line, error_location, self.message
                )
            }
            None => {
                write!(formatter, "[line {}] io:Error: {}", self.line, self.message)
            }
        }
    }
}

impl<'a> error::Error for InterpreterError<'a> {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        None
    }
}

impl<'a> InterpreterError<'a> {
    fn report(&self) {
        eprintln!("{}", self)
    }
}
pub fn error(line: u32, message: &str) {
    report(line, "", message);
}

fn report(line: u32, error_location: &str, message: &str) {
    eprintln!("[line {line}] io::Error {error_location}: {message}");
}
