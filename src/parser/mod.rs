pub mod parser_error;
use crate::expressions::expr::{Binary, Expr, Literal, Unary};
use crate::token::{LiteralValue, Token, TokenKind};
use crate::token_type::TokenType as TT;
use parser_error::ParserError;
use parser_error::ParserError::UnexpectedToken;
use std::iter::Peekable;
use std::vec::IntoIter;
use std::vec::Vec;

pub struct Parser<'a> {
    tokens: Peekable<IntoIter<Token<'a>>>,
}

impl<'a> Parser<'a> {
    pub fn new(tokens: Vec<Token<'a>>) -> Self {
        Parser {
            tokens: tokens.into_iter().peekable(),
        }
    }

    fn next_if_match(&mut self, token_types: &[TT]) -> Option<Token<'a>> {
        if self
            .tokens
            .peek()
            .is_some_and(|t| token_types.iter().any(|tt| &t.token_kind == tt))
        {
            self.tokens.next()
        } else {
            None
        }
    }

    fn next_token_type_if(&mut self, token_types: &'static [TT]) -> Option<TT> {
        if let Some(token) = self.tokens.peek() {
            for tt in token_types {
                if token.token_kind == *tt {
                    self.tokens.next();
                    return Some(*tt);
                }
            }
        }
        None
    }

    fn next_literal(
        &mut self,
        literal_types: &'static [TT],
    ) -> Result<LiteralValue<'a>, Option<Token<'a>>> {
        let token = self.tokens.next().ok_or(None)?;

        match token.token_kind {
            TokenKind::Value(literal_value)
                if literal_types.iter().any(|lt| literal_value == *lt) =>
            {
                Ok(literal_value)
            }
            _ => Err(Some(token)),
        }
    }

    pub fn expression(
        &mut self,
    ) -> Result<Box<dyn Expr + 'a>, ParserError<'a>> {
        self.equality()
    }

    pub fn equality(&mut self) -> Result<Box<dyn Expr + 'a>, ParserError<'a>> {
        static OPERATORS: [TT; 2] = [TT::BANG_EQUAL, TT::EQUAL_EQUAL];

        let mut expr = self.comparison()?;

        while let Some(operator) = self.next_token_type_if(&OPERATORS) {
            let rhs = self.comparison()?;
            expr = Box::new(Binary::new(expr, operator, rhs));
        }
        Ok(expr)
    }

    pub fn comparison(
        &mut self,
    ) -> Result<Box<dyn Expr + 'a>, ParserError<'a>> {
        static OPERATORS: [TT; 4] =
            [TT::GREATER, TT::GREATER_EQUAL, TT::LESS, TT::LESS_EQUAL];

        let mut expr = self.term()?;

        while let Some(operator) = self.next_token_type_if(&OPERATORS) {
            let rhs = self.term()?;
            expr = Box::new(Binary::new(expr, operator, rhs));
        }
        Ok(expr)
    }

    pub fn term(&mut self) -> Result<Box<dyn Expr + 'a>, ParserError<'a>> {
        static OPERATORS: [TT; 2] = [TT::MINUS, TT::PLUS];

        let mut expr = self.factor()?;

        while let Some(operator) = self.next_token_type_if(&OPERATORS) {
            let rhs = self.factor()?;
            expr = Box::new(Binary::new(expr, operator, rhs));
        }
        Ok(expr)
    }

    pub fn factor(&mut self) -> Result<Box<dyn Expr + 'a>, ParserError<'a>> {
        static OPERATORS: [TT; 2] = [TT::SLASH, TT::STAR];

        let mut expr = self.unary()?;

        while let Some(operator) = self.next_token_type_if(&OPERATORS) {
            let rhs = self.unary()?;
            expr = Box::new(Binary::new(expr, operator, rhs));
        }
        Ok(expr)
    }

    pub fn unary(&mut self) -> Result<Box<dyn Expr + 'a>, ParserError<'a>> {
        static OPERATORS: [TT; 2] = [TT::BANG, TT::MINUS];

        if let Some(operator) = self.next_token_type_if(&OPERATORS) {
            let value = self.unary()?;
            Ok(Box::new(Unary {
                operator,
                expr: value,
            }))
        } else {
            self.primary()
        }
    }

    pub fn primary(&mut self) -> Result<Box<dyn Expr + 'a>, ParserError<'a>> {
        static SIMPLE_LITERALS: [TT; 3] = [TT::FALSE, TT::TRUE, TT::NIL];

        match self.next_literal(&SIMPLE_LITERALS) {
            Ok(literal) => Ok(Box::new(Literal { value: literal })),
            Err(token) => Err(UnexpectedToken {
                token,
                expected_token_types: &SIMPLE_LITERALS,
            }),
        }
    }
}
