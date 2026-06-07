use crate::error_utils::StageError;
use std::borrow::ToOwned;

#[derive(Debug)]
pub struct ScannerError<'a> {
    pub line: usize,
    pub message: &'static str,
    pub error_location: Option<&'a str>,
}

impl<'a> From<ScannerError<'a>> for StageError {
    fn from(val: ScannerError<'a>) -> Self {
        Self {
            line: Some(val.line),
            message: val.message.to_owned(),
            error_location: val.error_location.map(ToOwned::to_owned),
            stage: "scanning",
            child: None,
        }
    }
}
