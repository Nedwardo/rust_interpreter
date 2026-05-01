use crate::token_type::TokenType as TT;
use core::fmt;
use core::fmt::{Debug, Display, Formatter};

#[allow(clippy::struct_field_names, reason = "Would otherwise be named type")]
#[derive(Copy, Clone, Debug)]
pub struct Token<'a> {
    pub kind: TT,
    pub token_value: Option<TokenValue<'a>>,
    pub line: usize,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum TokenValue<'a> {
    String(&'a str),
    Number(f64),
    False,
    True,
    Nil,
    Identifier(&'a str),
    Comment(&'a str),
}

impl<'a> Token<'a> {
    pub const fn new(kind: TT, line: usize) -> Self {
        Token {
            kind,
            token_value: None,
            line,
        }
    }

    pub const fn new_value(
        kind: TT,
        token_value: TokenValue<'a>,
        line: usize,
    ) -> Self {
        Token {
            kind,
            token_value: Some(token_value),
            line,
        }
    }
}

impl TokenValue<'static> {
    pub const fn from_keyword(token_type: TT) -> Option<Self> {
        match token_type {
            TT::FALSE => Some(Self::False),
            TT::TRUE => Some(Self::True),
            TT::NIL => Some(Self::Nil),
            _ => None,
        }
    }
}

impl Display for TokenValue<'_> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match *self {
            Self::String(value) => write!(f, r#""{value}""#),
            Self::Number(value) => write!(f, "{value}"),
            Self::False => write!(f, "false"),
            Self::True => write!(f, "true"),
            Self::Nil => write!(f, "nil"),
            Self::Identifier(name) => write!(f, "Var: {name}"),
            Self::Comment(comment) => write!(f, "// {comment}"),
        }
    }
}

#[cfg(test)]
impl PartialEq<&str> for TokenValue<'_> {
    fn eq(&self, other: &&str) -> bool {
        match self {
            Self::String(string) => other == string,
            _ => false,
        }
    }
}
