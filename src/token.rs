use crate::token_type::TokenType;
use std::fmt::{Display, Formatter, Result};

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum LiteralValue<'a> {
    String(&'a str),
    Number(f64),
}

impl<'a> Display for LiteralValue<'a> {
    fn fmt(&self, formatter: &mut Formatter) -> Result {
        let value: &dyn Display = match self {
            LiteralValue::String(value) => value,
            LiteralValue::Number(value) => value,
        };

        write!(formatter, "{}", value)
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Token<'a> {
    pub token_type: TokenType,
    pub lexeme: &'a str,
    pub literal: Option<LiteralValue<'a>>,
    pub line: usize,
}

impl<'a> Token<'a> {
    pub fn new(
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

impl<'a> Display for Token<'a> {
    fn fmt(&self, formatter: &mut Formatter) -> Result {
        write!(
            formatter,
            "{}, {}{}",
            self.token_type,
            self.lexeme,
            match &self.literal {
                Some(lit) => {
                    format!(", {}", lit)
                }
                None => {
                    "".to_string()
                }
            }
        )
    }
}
