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
    FailedToGenerateChildExpr {
        expr: &'static str,
        source: Box<Self>,
    },
}

impl ParserError<'_> {
    fn wrap(self, expr: &'static str) -> Self {
        Self::FailedToGenerateChildExpr {
            expr,
            source: Box::new(self),
        }
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
        }
    }
}

impl error::Error for ParserError<'_> {}
