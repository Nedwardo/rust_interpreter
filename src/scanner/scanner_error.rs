use std::error::Error;
use std::fmt;
use std::fmt::Write as _;
use std::fmt::{Display, Formatter};

#[derive(Debug)]
pub struct ScannerErrors {
    error_message: String,
}

#[derive(Debug)]
pub struct ScannerError<'a> {
    pub line: usize,
    pub message: &'static str,
    pub error_location: Option<&'a str>,
}

#[allow(unused, reason = "string writeln! cannot fail")]
impl<'a> ScannerErrors {
    pub fn new(errors: Vec<ScannerError<'a>>, source: &'a str) -> Self {
        let mut error_message = String::new();

        for err in errors {
            write!(
                &mut error_message,
                "{}\n\n",
                err.generate_error_message(source)
            );
        }

        error_message.truncate(error_message.len() - 1);

        Self { error_message }
    }
}

impl Display for ScannerErrors {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        Display::fmt(&self.error_message, f)
    }
}

impl ScannerError<'_> {
    fn generate_error_message(&self, source_string: &str) -> String {
        let source_line = source_string
            .split('\n')
            .nth(self.line - 1)
            .map_or("EOF", |ok| ok);

        self.error_location.map_or_else(
            || {
                format!(
                    "Error during scanning: {}\n {: >3} | {}",
                    self.message, self.line, source_line
                )
            },
            |error_location| {
                highlight_line_selection(
                    self.line,
                    source_line,
                    error_location,
                ).map_or_else(
                || format!("Errored generating the error message for {self:?}\nCouldn't find {error_location:?} in {source_line:?}")
                , |line_selection| format!(
                    "Error during scanning: {}\n{}",
                    self.message, line_selection
                )
                )
            },
        )
    }
}

fn highlight_line_selection(
    line_number: usize,
    line: &str,
    substr: &str,
) -> Option<String> {
    let start_index = line.find(substr)?;
    let substr_length = substr.chars().count();
    let carets = "^".repeat(substr_length);

    let substring_highlighter =
        format!("{carets:>width$}", width = start_index + substr_length);
    Some(format!(
        "{line_number:>4} | {line}\n     | {substring_highlighter}"
    ))
}

impl Error for ScannerErrors {}
