pub mod parser_error;
use crate::expressions::Expr;
use crate::expressions::Expr::{Binary, Grouping, Literal, Unary};
use crate::parser::parser_error::WrapErr;
use crate::token::TokenKind::SelfContained;
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

    pub fn parse(&mut self) -> Result<Box<Expr<'a>>, ParserError<'a>> {
        self.expression()
    }

    fn syncronise(&mut self) {
        while let Some(token) = self.tokens.next() {
            if matches!(token.token_kind, SelfContained(TT::SEMICOLON)) {
                break;
            }
            if self.tokens.peek().is_some_and(|next_token| {
                [
                    TT::CLASS,
                    TT::FUN,
                    TT::VAR,
                    TT::FOR,
                    TT::WHILE,
                    TT::PRINT,
                    TT::RETURN,
                ]
                .map(SelfContained)
                .contains(&next_token.token_kind)
            }) {
                break;
            }
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

    fn consume_literal(
        &mut self,
    ) -> Result<LiteralValue<'a>, Option<Token<'a>>> {
        let token = self.tokens.next().ok_or(None)?;

        match token.token_kind {
            TokenKind::Value(literal_value) => Ok(literal_value),
            _ => Err(Some(token)),
        }
    }

    pub fn expression(&mut self) -> Result<Box<Expr<'a>>, ParserError<'a>> {
        self.equality()
    }

    pub fn equality(&mut self) -> Result<Box<Expr<'a>>, ParserError<'a>> {
        static CTX: &str = "Equality";
        static OPERATORS: [TT; 2] = [TT::BANG_EQUAL, TT::EQUAL_EQUAL];

        let mut expr = self.comparison().wrap_err(CTX)?;

        while let Some(operator) = self.next_token_type_if(&OPERATORS) {
            let rhs = self.comparison().wrap_err(CTX)?;
            expr = Box::new(Binary {
                left: expr,
                operator,
                right: rhs,
            });
        }
        Ok(expr)
    }

    pub fn comparison(&mut self) -> Result<Box<Expr<'a>>, ParserError<'a>> {
        static CTX: &str = "Result";
        static OPERATORS: [TT; 4] =
            [TT::GREATER, TT::GREATER_EQUAL, TT::LESS, TT::LESS_EQUAL];

        let mut expr = self.term().wrap_err(CTX)?;

        while let Some(operator) = self.next_token_type_if(&OPERATORS) {
            let rhs = self.term().wrap_err(CTX)?;
            expr = Box::new(Binary {
                left: expr,
                operator,
                right: rhs,
            });
        }
        Ok(expr)
    }

    pub fn term(&mut self) -> Result<Box<Expr<'a>>, ParserError<'a>> {
        static CTX: &str = "Term";
        static OPERATORS: [TT; 2] = [TT::MINUS, TT::PLUS];

        let mut expr = self.factor().wrap_err(CTX)?;

        while let Some(operator) = self.next_token_type_if(&OPERATORS) {
            let rhs = self.factor().wrap_err(CTX)?;
            expr = Box::new(Binary {
                left: expr,
                operator,
                right: rhs,
            });
        }
        Ok(expr)
    }

    pub fn factor(&mut self) -> Result<Box<Expr<'a>>, ParserError<'a>> {
        static CTX: &str = "Factor";
        static OPERATORS: [TT; 2] = [TT::SLASH, TT::STAR];

        let mut expr = self.unary().wrap_err(CTX)?;

        while let Some(operator) = self.next_token_type_if(&OPERATORS) {
            let rhs = self.unary().wrap_err(CTX)?;
            expr = Box::new(Binary {
                left: expr,
                operator,
                right: rhs,
            });
        }
        Ok(expr)
    }

    pub fn unary(&mut self) -> Result<Box<Expr<'a>>, ParserError<'a>> {
        static CTX: &str = "Unary";
        static OPERATORS: [TT; 2] = [TT::BANG, TT::MINUS];

        if let Some(operator) = self.next_token_type_if(&OPERATORS) {
            let value = self.unary().wrap_err(CTX)?;
            Ok(Box::new(Unary {
                operator,
                expr: value,
            }))
        } else {
            self.primary()
        }
    }

    pub fn primary(&mut self) -> Result<Box<Expr<'a>>, ParserError<'a>> {
        static CTX: &str = "Primary";

        if self.next_token_type_if(&[TT::LEFT_PAREN]).is_some() {
            let expr = self.expression().wrap_err(CTX)?;

            match self.tokens.next() {
                Some(token) if token.token_kind == TT::RIGHT_PAREN => Ok(()),
                Some(token) => Err(Some(token)),
                None => Err(None),
            }
            .map_err(|token| UnexpectedToken {
                token,
                expected_token_types: &[TT::RIGHT_PAREN],
            })?;
            return Ok(Box::new(Grouping(expr)));
        }

        match self.consume_literal() {
            Ok(literal) => Ok(Box::new(Literal(literal))),
            Err(token) => Err(UnexpectedToken {
                token,
                expected_token_types: LiteralValue::token_types(),
            }),
        }
    }
}
