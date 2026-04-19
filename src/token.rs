use crate::token_type::TokenType as TT;
use core::fmt;
use core::fmt::{Debug, Display, Formatter};

#[allow(clippy::struct_field_names, reason = "Would otherwise be named type")]
#[cfg_attr(test, derive(Copy, Clone))]
#[derive(Debug)]
pub struct Token<'a> {
    pub token_kind: TokenKind<'a>,
    pub line: usize,
}

#[cfg_attr(test, derive(Copy, Clone))]
#[derive(Debug, PartialEq)]
pub enum TokenKind<'a> {
    Value(LiteralValue<'a>),
    Identifier(&'a str),
    SelfContained(TT),
    Comment(&'a str),
}

#[cfg_attr(test, derive(Copy, Clone))]
#[derive(Debug, PartialEq)]
pub enum LiteralValue<'a> {
    String(&'a str),
    Number(f64),
    False,
    True,
    Nil,
}

impl<'a> Token<'a> {
    pub const fn new(token_kind: TokenKind<'a>, line: usize) -> Self {
        Token { token_kind, line }
    }
}

impl PartialEq<TT> for TokenKind<'_> {
    fn eq(&self, other: &TT) -> bool {
        match self {
            Self::Value(literal_value) => *literal_value == *other,
            Self::Identifier(..) => *other == TT::IDENTIFIER,
            Self::SelfContained(token_type) => other == token_type,
            Self::Comment(..) => *other == TT::COMMENT,
        }
    }
}

impl Display for TokenKind<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Value(literal_value) => {
                write!(f, "Value {literal_value}")
            }
            Self::Identifier(name) => {
                write!(f, "Identifier {name}")
            }
            Self::SelfContained(token_type) => {
                write!(f, "SelfContained {token_type}")
            }
            Self::Comment(comment) => {
                write!(f, "Comment {comment}")
            }
        }
    }
}

impl LiteralValue<'_> {
    pub const fn from_keyword(token_type: TT) -> Option<LiteralValue<'static>> {
        match token_type {
            TT::FALSE => Some(LiteralValue::False),
            TT::TRUE => Some(LiteralValue::True),
            TT::NIL => Some(LiteralValue::Nil),
            _ => None,
        }
    }
    pub const fn token_types() -> &'static [TT] {
        static TOKEN_TYPES: [TT; 5] =
            [TT::STRING, TT::NUMBER, TT::FALSE, TT::TRUE, TT::NIL];

        &TOKEN_TYPES
    }
}

impl PartialEq<TT> for LiteralValue<'_> {
    fn eq(&self, other: &TT) -> bool {
        match self {
            Self::String(_) => *other == TT::STRING,
            Self::Number(_) => *other == TT::NUMBER,
            Self::False => *other == TT::FALSE,
            Self::True => *other == TT::TRUE,
            Self::Nil => *other == TT::NIL,
        }
    }
}

impl Display for LiteralValue<'_> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match *self {
            LiteralValue::String(value) => write!(f, "{value}"),
            LiteralValue::Number(value) => write!(f, "{value}"),
            LiteralValue::False => write!(f, "false"),
            LiteralValue::True => write!(f, "true"),
            LiteralValue::Nil => write!(f, "nil"),
        }
    }
}

#[cfg(test)]
impl PartialEq<&str> for LiteralValue<'_> {
    fn eq(&self, other: &&str) -> bool {
        match self {
            Self::String(string) => other == string,
            _ => false,
        }
    }
}
