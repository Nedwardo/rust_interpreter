use crate::error_utils::StageError;
use crate::token::Token;
use crate::token_type::TokenType;
use std::fmt::Display;

#[derive(Debug, Clone)]
pub enum ParserError {
    UnexpectedToken {
        source: String,
        line: usize,
        token_type: TokenType,
        expected_token_types: &'static [TokenType],
    },
    InvalidAssignmentTarget {
        line: usize,
    },
    EOFWhileExpecting {
        expected_token_types: &'static [TokenType],
    },
    UnexpectedEOF,
    FailedToGenerateChildExpr {
        error_message: String,
        source: Vec<Self>,
    },
}

impl ParserError {
    pub fn unexpected_token(
        token: &Token<'_>,
        expected_token_types: &'static [TokenType],
    ) -> Self {
        Self::UnexpectedToken {
            source: token.token_value.map_or_else(
                || token.kind.to_string(),
                |token| token.to_string(),
            ),
            line: token.line,
            token_type: token.kind,
            expected_token_types,
        }
    }

    pub const fn expected_token(
        expected_token_types: &'static [TokenType],
    ) -> Self {
        Self::EOFWhileExpecting {
            expected_token_types,
        }
    }

    pub fn wrap(self, error_message: String) -> Self {
        Self::FailedToGenerateChildExpr {
            error_message,
            source: vec![self],
        }
    }
}

pub trait WrapErr<T, E, D>
where
    D: Display + Send + Sync + 'static,
{
    fn wrap_err(self, expr: D) -> Result<T, E>;

    fn wrap_err_with<F: FnOnce() -> D>(self, f: F) -> Result<T, E>
    where
        Self: Sized,
    {
        self.wrap_err(f())
    }
}

impl<T> WrapErr<T, ParserError, String> for Result<T, ParserError> {
    fn wrap_err(self, expr: String) -> Self {
        self.map_err(|e| e.wrap(expr))
    }
}

impl<T> WrapErr<T, ParserError, String> for Result<T, Vec<ParserError>> {
    fn wrap_err(self, expr: String) -> Result<T, ParserError> {
        self.map_err(|e| ParserError::FailedToGenerateChildExpr {
            error_message: expr,
            source: e,
        })
    }
}

impl From<ParserError> for StageError {
    fn from(val: ParserError) -> Self {
        match val {
            ParserError::UnexpectedToken {
                source,
                line,
                token_type,
                expected_token_types,
            } => Self {
                line: Some(line),
                message: format!(
                    "Unexpected Token: Expected one of: {expected_token_types:?}, found {token_type}",
                ),
                error_location: Some(source),
                stage: "parsing",
                children: Vec::new(),
            },
            ParserError::InvalidAssignmentTarget { line } => Self {
                line: Some(line),
                message: "Invalid assignment target".to_owned(),
                error_location: Some("=".to_owned()),
                stage: "parsing",
                children: Vec::new(),
            },
            ParserError::EOFWhileExpecting {
                expected_token_types,
            } => Self {
                line: None,
                message: format!(
                    "Found EOF while expecting {expected_token_types:?}"
                ),
                error_location: None,
                stage: "parsing",
                children: Vec::new(),
            },
            ParserError::UnexpectedEOF => Self {
                line: None,
                message: "Unexpected EOF".to_owned(),
                error_location: None,
                stage: "parsing",
                children: Vec::new(),
            },
            ParserError::FailedToGenerateChildExpr {
                error_message,
                source,
            } => Self {
                line: None,
                message: error_message,
                error_location: None,
                stage: "parsing",
                children: source.iter().map(|e| e.clone().into()).collect(),
            },
        }
    }
}
