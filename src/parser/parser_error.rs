use crate::error_utils::StageError;
use crate::expressions::ValueError;
use crate::token::{Span, Token};
use crate::token_type::TokenType;

#[derive(Debug, Clone)]
pub struct ParserError {
    pub kind: ParserErrorKind,
    pub synchronise: bool,
}

#[derive(Debug, Clone)]
pub enum ParserErrorKind {
    UnexpectedToken {
        span: Span,
        token_type: TokenType,
        expected_token_types: Vec<TokenType>,
    },
    InvalidAssignmentTarget {
        span: Span,
    },
    EOFWhileExpecting {
        expected_token_types: Vec<TokenType>,
    },
    UnexpectedEOF {
        expected: &'static str,
    },
    TooManyArguments {
        span: Span,
    },
    BlockError {
        errors: Vec<ParserError>,
    },
    ValueError(ValueError),
}

impl ParserError {
    pub fn unexpected_token(token: &Token, token_types: &[TokenType]) -> Self {
        Self {
            kind: ParserErrorKind::UnexpectedToken {
                span: token.span,
                token_type: token.token_kind,
                expected_token_types: token_types.to_owned(),
            },
            synchronise: true,
        }
    }

    pub const fn invalid_assignment_target(span: Span) -> Self {
        Self {
            kind: ParserErrorKind::InvalidAssignmentTarget { span },
            synchronise: true,
        }
    }

    pub const fn unexpected_eof(expected: &'static str) -> Self {
        Self {
            kind: ParserErrorKind::UnexpectedEOF { expected },
            synchronise: true,
        }
    }

    pub const fn too_many_arguments(span: Span, synchronise: bool) -> Self {
        Self {
            kind: ParserErrorKind::TooManyArguments { span },
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

    pub const fn value_error(error: ValueError) -> Self {
        Self {
            kind: ParserErrorKind::ValueError(error),
            synchronise: false,
        }
    }
}

impl From<ParserError> for StageError {
    fn from(val: ParserError) -> Self {
        match val.kind {
            ParserErrorKind::UnexpectedToken {
                span,
                token_type,
                expected_token_types,
            } => Self {
                span: Some(span),
                message: format!(
                    "Unexpected Token: Expected one of: {}, found {token_type}",
                    expected_token_types
                        .iter()
                        .map(|c| format!("'{c}'"))
                        .collect::<Vec<_>>()
                        .join(", ")
                ),
                stage: "parsing",
                children: Vec::new(),
            },
            ParserErrorKind::InvalidAssignmentTarget { span } => Self {
                span: Some(span),
                message: "Invalid assignment target".to_owned(),
                stage: "parsing",
                children: Vec::new(),
            },
            ParserErrorKind::EOFWhileExpecting {
                expected_token_types,
            } => Self {
                span: None,
                message: format!(
                    "Found EOF while expecting {expected_token_types:?}"
                ),
                stage: "parsing",
                children: Vec::new(),
            },
            ParserErrorKind::UnexpectedEOF { expected } => Self {
                span: None,
                message: format!("Unexpected EOF, while parsing {expected}"),
                stage: "parsing",
                children: Vec::new(),
            },
            ParserErrorKind::TooManyArguments { span } => Self {
                span: Some(span),
                message: "Can't have more than 255 arguments.".to_owned(),
                stage: "Parsing",
                children: Vec::new(),
            },
            ParserErrorKind::BlockError { errors } => Self {
                span: None,
                message: "Error while generating block".to_owned(),
                stage: "parsing",
                children: errors
                    .iter()
                    .map(|e| Self::from(e.clone()))
                    .collect(),
            },
            ParserErrorKind::ValueError(ValueError::NotAValue(span)) => Self {
                span: Some(span),
                message: "Expected a value".to_owned(),
                stage: "parsing",
                children: vec![],
            },

            ParserErrorKind::ValueError(ValueError::NotANumber(span)) => Self {
                span: Some(span),
                message: "Value is not a number".to_owned(),
                stage: "parsing",
                children: vec![],
            },
        }
    }
}
