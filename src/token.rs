use crate::token_type::TokenType;
use core::fmt::{Display, Formatter, Result};

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum LiteralValue<'a> {
    String(&'a str),
    Number(f64),
}

impl Display for LiteralValue<'_> {
    fn fmt(&self, f: &mut Formatter) -> Result {
        match *self {
            LiteralValue::String(value) => write!(f, "{value}"),
            LiteralValue::Number(value) => write!(f, "{value}"),
        }
    }
}

#[allow(clippy::struct_field_names, reason = "Would otherwise be named type")]
#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Token<'a> {
    pub token_type: TokenType,
    pub lexeme: &'a str,
    pub literal: Option<LiteralValue<'a>>,
    pub line: usize,
}

impl<'a> Token<'a> {
    pub const fn new(
        token_type: TokenType,
        lexeme: &'a str,
        literal: Option<LiteralValue<'a>>,
        line: usize,
    ) -> Self {
        Token {
            token_type,
            lexeme,
            literal,
            line,
        }
    }
}

impl Display for Token<'_> {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(
            f,
            "{}, {}{}",
            self.token_type,
            self.lexeme,
            self.literal.map_or(String::new(), |lit| format!(", {lit}"))
        )
    }
}
