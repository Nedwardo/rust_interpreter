use crate::error_utils::StageError;

#[derive(Debug)]
pub struct ScannerError<'a> {
    pub line: usize,
    pub message: &'static str,
    pub error_location: Option<&'a str>,
}

impl<'a> From<ScannerError<'a>> for StageError<'a> {
    fn from(val: ScannerError<'a>) -> Self {
        StageError {
            line: Some(val.line),
            message: val.message.to_owned(),
            error_location: val.error_location,
            stage: "scanning",
            child: None,
        }
    }
}
