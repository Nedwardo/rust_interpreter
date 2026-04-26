use crate::token::Token;
use crate::token_type::TokenType;
use std::error;
use std::fmt;
use std::fmt::{Display, Formatter};

#[derive(Debug)]
pub enum ParserError<'a> {
    UnexpectedToken {
        token: Option<Token<'a>>,
        expected_token_types: &'static [TokenType],
    },
    UnexpectedEOF,
    FailedToGenerateChildExpr {
        expr: String,
        source: Box<Self>,
    },
}

impl ParserError<'_> {
    pub fn wrap(self, expr: String) -> Self {
        Self::FailedToGenerateChildExpr {
            expr,
            source: Box::new(self),
        }
    }
}

pub trait WrapErr<'a, T> {
    fn wrap_err(self, expr: String) -> Result<T, ParserError<'a>>;
}

impl<'a, T> WrapErr<'a, T> for Result<T, ParserError<'a>> {
    fn wrap_err(self, expr: String) -> Self {
        self.map_err(|e| e.wrap(expr))
    }
}

impl Display for ParserError<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match &self {
            Self::UnexpectedToken {
                token: Some(token),
                expected_token_types,
            } => write!(
                f,
                "Unexpected token: {token:?}, expected one of token types: {expected_token_types:?}"
            ),
            Self::UnexpectedToken {
                token: None,
                expected_token_types,
            } => write!(
                f,
                "Unexpected EOF, while looking for {expected_token_types:?}"
            ),
            Self::FailedToGenerateChildExpr { expr, source } => {
                write!(
                    f,
                    "Failed to genererate Expr {expr:?}, cause:\n{source}"
                )
            }
            Self::UnexpectedEOF => {
                write!(f, "Unexpected EOF")
            }
        }
    }
}

impl error::Error for ParserError<'_> {}
