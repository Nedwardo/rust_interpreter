use crate::token_type::TokenType;
use std::fmt::{Display, Formatter, Result};

pub enum LiteralValue {
    String(String),
    Number(f64),
}

impl Display for LiteralValue {
    fn fmt(&self, formatter: &mut Formatter) -> Result {
        let value: &dyn Display = match self {
            LiteralValue::String(value) => value,
            LiteralValue::Number(value) => value,
        };

        write!(formatter, "{}", value)
    }
}

pub struct Token<'token_lifetime> {
    pub token_type: TokenType,
    pub lexeme: &'token_lifetime str,
    pub literal: Option<LiteralValue>,
    pub line: usize,
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
