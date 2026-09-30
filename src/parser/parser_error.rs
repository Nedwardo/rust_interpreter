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
    InvalidAssignmentTarget(Span),
    EOFWhileExpecting(Vec<TokenType>),
    UnexpectedEOF(&'static str),
    TooManyArguments(Span),
    BlockError(Vec<ParserError>),
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
            kind: ParserErrorKind::InvalidAssignmentTarget(span),
            synchronise: true,
        }
    }

    pub const fn unexpected_eof(expected: &'static str) -> Self {
        Self {
            kind: ParserErrorKind::UnexpectedEOF(expected),
            synchronise: true,
        }
    }

    pub const fn too_many_arguments(span: Span, synchronise: bool) -> Self {
        Self {
            kind: ParserErrorKind::TooManyArguments(span),
            synchronise,
        }
    }

    pub fn expected_token(expected_token_types: &[TokenType]) -> Self {
        Self {
            kind: ParserErrorKind::EOFWhileExpecting(
                expected_token_types.to_owned(),
            ),
            synchronise: true,
        }
    }
    pub fn block_error(errors: Vec<Self>) -> Self {
        let synchronise = !errors.iter().all(|val| !val.synchronise);
        Self {
            kind: ParserErrorKind::BlockError(errors),
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

impl ParserErrorKind {
    const fn span(&self) -> Option<Span> {
        match self {
            Self::UnexpectedToken { span, .. }
            | Self::InvalidAssignmentTarget(span)
            | Self::TooManyArguments(span)
            | Self::ValueError(
                ValueError::NotAValue(span) | ValueError::NotANumber(span),
            ) => Some(*span),

            Self::BlockError(..)
            | Self::UnexpectedEOF(..)
            | Self::EOFWhileExpecting(..) => None,
        }
    }

    fn message(&self) -> String {
        match self {
            Self::UnexpectedToken {
                token_type,
                expected_token_types,
                ..
            } => format!(
                "Unexpected Token: Expected one of: {}, found {token_type}",
                expected_token_types
                    .iter()
                    .map(|c| format!("'{c}'"))
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            Self::InvalidAssignmentTarget(..) => {
                "Invalid assignment target".to_owned()
            }
            Self::EOFWhileExpecting(expected_token_types) => {
                format!("Found EOF while expecting {expected_token_types:?}")
            }
            Self::UnexpectedEOF(expected) => {
                format!("Unexpected EOF, while parsing {expected}")
            }
            Self::TooManyArguments(..) => {
                "Can't have more than 255 arguments.".to_owned()
            }
            Self::BlockError(..) => "Error while generating block".to_owned(),
            Self::ValueError(ValueError::NotAValue(..)) => {
                "Expected a value".to_owned()
            }
            Self::ValueError(ValueError::NotANumber(..)) => {
                "Value is not a number".to_owned()
            }
        }
    }

    fn children(&self) -> Vec<ParserError> {
        if let Self::BlockError(errors) = self {
            errors.clone()
        } else {
            Vec::new()
        }
    }
}

impl From<ParserError> for StageError {
    fn from(val: ParserError) -> Self {
        Self {
            span: val.kind.span(),
            message: val.kind.message(),
            stage: "Parsing",
            children: val.kind.children().into_iter().map(Into::into).collect(),
        }
    }
}
