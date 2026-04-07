use crate::expressions::expr::{Expr, Binary};
use std::vec::Vec;
use std::vec;
use crate::token::Token;
use crate::token_type::TokenType;
use crate::token_type::TokenType::{BANG_EQUAL, EQUAL_EQUAL};

pub struct Parser<'parser_lt, 'token_lt>{
    tokens: Vec<Token<'token_lt>>,
    current: usize
}

pub fn build_parser<'token_lt, 'parser_lt>(tokens: Vec<Token<'token_lt>>) -> Parser<'token_lt>
where
    'token_lt: 'parser_lt,
{
    Parser {
        tokens,
        current: 0
    }
}

impl<'parser_lt, 'token_lt> Parser<'parser_lt, 'token_lt> {
    pub fn expression(&self) -> &dyn Expr{
        return self.equality()
    }

    pub fn equality(&self) -> Expr {
        let expr = self.comparison();

        while self.match_token(vec![BANG_EQUAL, EQUAL_EQUAL]) {
            let operator = self.previous();
            let right = self.comparison();
            let expr = Binary {
                expr, operator, right
            };
        }

    expr
  }

    fn match_token(&self, token_types: Vec<TokenType>) -> bool {
        for token_type in token_types.iter() {
            if self.check(token_type) {
                self.advance();
                return true;
            }
        }
        false
    }

    fn check(&self, token_type: &TokenType) -> bool {
        if self.isAtEnd() {
            return false;
        }
    self.peek().token_type == token_type
    }
  }
