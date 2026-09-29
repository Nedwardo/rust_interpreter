use crate::{error_utils::StageError, token::Span};
use std::borrow::ToOwned;

#[derive(Debug)]
pub struct ScannerError {
    pub message: &'static str,
    pub error_location: Span,
}

impl From<ScannerError> for StageError {
    fn from(val: ScannerError) -> Self {
        Self {
            span: Some(val.error_location),
            message: val.message.to_owned(),
            stage: "scanning",
            children: Vec::new(),
        }
    }
}
