use std::error;
use std::fmt::{Display, Formatter, Result};
use std::io;
use std::path::Path;

#[derive(Debug)]
pub struct ReadFileError {
    pub path: Box<Path>,
    pub source: io::Error,
}

impl Display for ReadFileError {
    fn fmt(&self, formatter: &mut Formatter) -> Result {
        write!(formatter, "Unable to read file at {}", self.path.display())
    }
}

impl error::Error for ReadFileError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        Some(&self.source)
    }
}
