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

fn get_line_span(string: &str, index: usize) -> Span {
    let start = string[..index].rfind('\n').map_or(0, |i| i + 1);
    let end = string[index..]
        .find('\n')
        .map_or(string.len(), |i| index + i);
    (start, end)
}

impl StageError {
    fn generate_error_message(&self, source_string: &str) -> String {
        let mut formatted_error_message = if self.message.is_empty() {
            String::new()
        } else if let Some(span) = self.span {
            let error_line_number = source_string[..span.0]
                .chars()
                .filter(|c| *c == '\n')
                .count()
                + 1;
            let error_line = get_line_span(source_string, span.0);

            let line_selection = highlight_line_selection(
                source_string,
                error_line_number,
                error_line,
                span,
            );
            format!(
                "Error during {}: {}\n{line_selection}",
                self.stage, self.message
            )
        } else {
            format!("Error during {}: {}", self.stage, self.message)
        };

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
    source: &str,
    line_number: usize,
    line_span: Span,
    highlight_span: Span,
) -> String {
    let line = &source[line_span.0..line_span.1];
    let offset = highlight_span.0 - line_span.0;
    let substr_length = highlight_span.1 - highlight_span.0;

    let carets = "^".repeat(substr_length);
    let pre_spacing = " ".repeat(offset);

    format!("{line_number:>4} | {line}\n     | {pre_spacing}{carets}",)
}
