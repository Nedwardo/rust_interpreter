use std::error::Error;
use std::fmt;
use std::fmt::Debug;
use std::fmt::Write as _;
use std::fmt::{Display, Formatter};

use crate::token::Span;

#[derive(Debug)]
pub struct StageError {
    pub span: Option<Span>,
    pub message: String,
    pub stage: &'static str,
    pub children: Vec<Self>,
}

fn get_line_from_span(string: &str, index: usize) -> &str {
    let start = string[..index].rfind('\n').map_or(0, |i| i + 1);
    let end = string[index..]
        .find('\n')
        .map_or(string.len(), |i| index + i);
    &string[start..end]
}

impl StageError {
    fn generate_error_message(&self, source_string: &str) -> String {
        let mut formatted_error_message = self.span.map_or_else(
            || format!("Error during {}: {}", self.stage, self.message),
            |span| {

        let error_source = &source_string[span.0..span.1];
        let error_line_number = source_string[..span.0]
            .chars()
            .filter(|c| *c == '\n')
            .count() + 1;
        let error_line = get_line_from_span(source_string, span.0);

        highlight_line_selection(
                    error_line_number,
                    error_line,
                    error_source,
                ).map_or_else(
                || format!("Errored generating the error message for {self:?}\nCouldn't find {error_source:?} in {error_line:?}")
                , |line_selection| format!(
                    "Error during {}: {}\n{}",
                    self.stage, self.message, line_selection
                )
                )
            });

        for error_source in &self.children {
            formatted_error_message.push('\n');
            formatted_error_message
                .push_str(&error_source.generate_error_message(source_string));
        }
        formatted_error_message
    }
}

pub struct HydratedStageError {
    error_message: String,
}

#[allow(unused, reason = "string write! cannot fail")]
impl HydratedStageError {
    pub fn hydrate_errors(
        errors: Vec<impl Into<StageError>>,
        source: &str,
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

    pub fn hydrate_error(error: &StageError, source: &str) -> Self {
        Self {
            error_message: error.generate_error_message(source),
        }
    }
}

impl Display for HydratedStageError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        Display::fmt(&self.error_message, f)
    }
}

impl Debug for HydratedStageError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        Display::fmt(&self.error_message, f)
    }
}

impl Error for HydratedStageError {}

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
