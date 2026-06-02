use std::error::Error;
use std::fmt;
use std::fmt::Write as _;
use std::fmt::{Display, Formatter};

#[derive(Debug)]
pub struct StageError<'a> {
    pub line: Option<usize>,
    pub message: String,
    pub error_location: Option<&'a str>,
    pub stage: &'static str,
    pub child: Option<Box<Self>>,
}

impl StageError<'_> {
    fn generate_error_message(&self, source_string: &str) -> String {
        let mut formatted_error_message = self.line.map_or_else(
            || format!("Error during {}: {}", self.stage, self.message),
            |line| {
                let source_line =
                source_string.split('\n').nth(line - 1).unwrap_or("EOF");

            self.error_location.map_or_else(
            || {
                format!(
                    "Error during {}: {}\n {: >3} | {}",
                    self.stage, self.message, line, source_line
                )
            },
            |error_location| {
                highlight_line_selection(
                    line,
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
        )});

        if let Some(error_source) = &self.child {
            formatted_error_message.push('\n');
            formatted_error_message
                .push_str(&error_source.generate_error_message(source_string));
        }
        formatted_error_message
    }
}

#[derive(Debug)]
pub struct FlattenedError {
    error_message: String,
}

impl<'a> FlattenedError {
    pub fn flatten(
        errors: Vec<impl Into<StageError<'a>>>,
        source: &'a str,
    ) -> Self {
        let mut error_message = String::new();

        for err in errors {
            write!(
                &mut error_message,
                "{}\n\n",
                err.into().generate_error_message(source)
            );
        }

        error_message.truncate(error_message.len() - 1);

        Self { error_message }
    }
}

impl Display for FlattenedError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        Display::fmt(&self.error_message, f)
    }
}

impl Error for FlattenedError {}

pub fn highlight_line_selection(
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
