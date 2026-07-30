use crate::error_utils::StageError;
use crate::token::Token;
use crate::token_type::TokenType;

#[derive(Debug, Clone)]
pub struct ParserError {
    pub kind: ParserErrorKind,
    pub synchronise: bool,
}

#[derive(Debug, Clone)]
pub enum ParserErrorKind {
    UnexpectedToken {
        source: String,
        line: usize,
        token_type: TokenType,
        expected_token_types: Vec<TokenType>,
    },
    InvalidAssignmentTarget {
        line: usize,
    },
    EOFWhileExpecting {
        expected_token_types: Vec<TokenType>,
    },
    UnexpectedEOF {
        expected: &'static str,
    },
    TooManyArguments {
        line: usize,
    },
    BlockError {
        errors: Vec<ParserError>,
    },
}

impl ParserError {
    pub fn unexpected_token(
        token: &Token<'_>,
        token_types: &[TokenType],
    ) -> Self {
        Self {
            kind: ParserErrorKind::UnexpectedToken {
                source: token.token_value.map_or_else(
                    || token.kind.to_string(),
                    |token| token.to_string(),
                ),
                line: token.line,
                token_type: token.kind,
                expected_token_types: token_types.to_owned(),
            },
            synchronise: true,
        }
    }

    pub const fn invalid_assignment_target(line: usize) -> Self {
        Self {
            kind: ParserErrorKind::InvalidAssignmentTarget { line },
            synchronise: true,
        }
    }

    pub const fn unexpected_eof(expected: &'static str) -> Self {
        Self {
            kind: ParserErrorKind::UnexpectedEOF { expected },
            synchronise: true,
        }
    }

    pub const fn too_many_arguments(line: usize, synchronise: bool) -> Self {
        Self {
            kind: ParserErrorKind::TooManyArguments { line },
            synchronise,
        }
    }

    pub fn expected_token(expected_token_types: &[TokenType]) -> Self {
        Self {
            kind: ParserErrorKind::EOFWhileExpecting {
                expected_token_types: expected_token_types.to_owned(),
            },
            synchronise: true,
        }
    }
    pub fn block_error(errors: Vec<Self>) -> Self {
        let synchronise = !errors.iter().all(|val| !val.synchronise);
        Self {
            kind: ParserErrorKind::BlockError { errors },
            synchronise,
        }
    }
}

impl From<ParserError> for StageError {
    fn from(val: ParserError) -> Self {
        match val.kind {
            ParserErrorKind::UnexpectedToken {
                source,
                line,
                token_type,
                expected_token_types,
            } => Self {
                line: Some(line),
                message: format!(
                    "Unexpected Token: Expected one of: {}, found {token_type}",
                    expected_token_types
                        .iter()
                        .map(|c| format!("'{c}'"))
                        .collect::<Vec<_>>()
                        .join(", ")
                ),
                error_location: Some(source),
                stage: "parsing",
                children: Vec::new(),
            },
            ParserErrorKind::InvalidAssignmentTarget { line } => Self {
                line: Some(line),
                message: "Invalid assignment target".to_owned(),
                error_location: Some("=".to_owned()),
                stage: "parsing",
                children: Vec::new(),
            },
            ParserErrorKind::EOFWhileExpecting {
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
            ParserErrorKind::UnexpectedEOF { expected } => Self {
                line: None,
                message: format!("Unexpected EOF, while parsing {expected}"),
                error_location: None,
                stage: "parsing",
                children: Vec::new(),
            },
            ParserErrorKind::TooManyArguments { line } => Self {
                line: Some(line),
                message: "Can't have more than 255 arguments.".to_owned(),
                error_location: None,
                stage: "Parsing",
                children: Vec::new(),
            },
            ParserErrorKind::BlockError { errors } => Self {
                line: None,
                message: "Error while generating block".to_owned(),
                error_location: None,
                stage: "parsing",
                children: errors
                    .iter()
                    .map(|e| Self::from(e.clone()))
                    .collect(),
            },
        }
    }
}
