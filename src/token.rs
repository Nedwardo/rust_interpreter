use crate::token_type::TokenType as TT;
use core::fmt::Debug;

pub type Span = (usize, usize);

#[derive(Copy, Clone, Debug)]
pub struct Token {
    pub token_kind: TT,
    pub span: Span,
}

impl Token {
    pub const fn new(token_kind: TT, span: Span) -> Self {
        Self { token_kind, span }
    }
}
