use crate::token::Token;
use crate::token_type::TokenType;
use std::error;
use std::fmt;
use std::fmt::{Display, Formatter};

#[derive(Debug)]
pub enum ParserError {
    UnexpectedToken {
        token: OwnedToken,
        expected_token_types: &'static [TokenType],
    },
    EOFWhileExpecting {
        expected_token_types: &'static [TokenType],
    },
    UnexpectedEOF,
    FailedToGenerateChildExpr {
        error_message: String,
        source: Box<Self>,
    },
}

#[derive(Debug)]
pub struct OwnedToken {
    pub lexeme: String,
    pub kind: TokenType,
    pub line: usize,
}

impl ParserError {
    pub fn unexpected_token(
        token: &Token,
        expected_token_types: &'static [TokenType],
    ) -> Self {
        Self::UnexpectedToken {
            token: OwnedToken::from(*token),
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
            source: Box::new(self),
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

impl Display for ParserError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Self::UnexpectedToken {
                token,
                expected_token_types,
            } => write!(
                f,
                "Line {}\nParse Error: Expected one of: {expected_token_types:?}, found {}",
                token.line, token.lexeme
            ),
            Self::EOFWhileExpecting {
                expected_token_types,
            } => write!(
                f,
                "Parse Error: Found EOF while expecting {expected_token_types:?}",
            ),
            Self::FailedToGenerateChildExpr {
                error_message,
                source,
            } => {
                write!(f, "{error_message}\n{source}",)
            }
            Self::UnexpectedEOF => {
                write!(f, "Unexpected EOF")
            }
        }
    }
}

impl error::Error for ParserError {}

impl From<Token<'_>> for OwnedToken {
    fn from(value: Token<'_>) -> Self {
        Self {
            lexeme: value.token_value.map_or_else(
                || value.kind.to_string(),
                |token| token.to_string(),
            ),
            kind: value.kind,
            line: value.line,
        }
    }
}
