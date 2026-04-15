use core::fmt::{Display, Formatter, Result};
use std::error;

#[derive(Debug)]
pub struct InterpreterError<'a> {
    pub line: usize,
    pub message: &'static str,
    pub error_location: Option<&'a str>,
}

impl Display for InterpreterError<'_> {
    fn fmt(&self, f: &mut Formatter) -> Result {
        match self.error_location {
            Some(error_location) => {
                write!(
                    f,
                    "[line {}] io::Error {}: {}",
                    self.line, error_location, self.message
                )
            }
            None => {
                write!(f, "[line {}] io:Error: {}", self.line, self.message)
            }
        }
    }
}

impl error::Error for InterpreterError<'_> {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        None
    }
}

impl InterpreterError<'_> {
    fn report(&self) {
        eprintln!("{self}");
    }
}
pub fn error(line: u32, message: &str) {
    report(line, "", message);
}

fn report(line: u32, error_location: &str, message: &str) {
    eprintln!("[line {line}] io::Error {error_location}: {message}");
}
