pub mod parser_error;
use crate::error_utils::StageError;
use crate::expressions::BinaryOperator as BinaryOp;
use crate::expressions::{Expr, Statment, UnaryOperator, Value};
use crate::parser::parser_error::{ParserError, WrapErr};
use crate::token::Token;
use crate::token::TokenValue as TV;
use crate::token_type::TokenType as TT;
use std::vec::IntoIter;
use std::vec::Vec;

pub const fn prefix_precedence(token_type: UnaryOperator) -> usize {
    match token_type {
        UnaryOperator::BANG | UnaryOperator::MINUS => 13,
    }
}

pub const fn infix_precedence(token_type: BinaryOp) -> (usize, usize) {
    match token_type {
        BinaryOp::COMMA => (1, 2),
        BinaryOp::QUESTION_MARK | BinaryOp::COLON => (4, 3),
        BinaryOp::EQUAL_EQUAL | BinaryOp::BANG_EQUAL => (5, 6),
        BinaryOp::LESS
        | BinaryOp::LESS_EQUAL
        | BinaryOp::GREATER
        | BinaryOp::GREATER_EQUAL => (7, 8),
        BinaryOp::PLUS | BinaryOp::MINUS => (9, 10),
        BinaryOp::STAR | BinaryOp::SLASH => (11, 12),
    }
}

pub struct Parser<'a> {
    tokens: IntoIter<Token<'a>>,
    peeked: Option<Token<'a>>,
}

pub fn parse(
    tokens: Vec<Token>,
) -> Result<Vec<Statment>, Vec<impl Into<StageError>>> {
    Parser::new(tokens).parse()
}

impl<'a> Parser<'a> {
    pub fn new(tokens: Vec<Token<'a>>) -> Self {
        Parser {
            tokens: tokens.into_iter(),
            peeked: None,
        }
    }

    fn next(&mut self) -> Option<Token<'a>> {
        if self.peeked.is_some() {
            return self.peeked.take();
        }
        self.tokens.by_ref().find(|token| token.kind != TT::COMMENT)
    }

    fn peek(&mut self) -> Option<&Token<'a>> {
        self.peeked = self.next();
        self.peeked.as_ref()
    }

    fn parse(&mut self) -> Result<Vec<Statment<'a>>, Vec<ParserError>> {
        let mut statments = Vec::new();
        let mut errors = Vec::new();
        while let Some(next_token) = self.peek() {
            let statment = if next_token.kind == TT::VAR {
                self.next();
                self.declaration()
            } else if next_token.kind == TT::PRINT {
                self.next();
                self.expression(0).map(Statment::Print)
            } else {
                self.expression(0).map(Statment::Expression)
            };

            if statment.is_err() {
                self.synchronise();
            }

            match statment {
                Ok(s) => statments.push(s),
                Err(e) => errors.push(e),
            }

            if let Some(token) = self.next()
                && token.kind != TT::SEMICOLON
            {
                errors.push(ParserError::unexpected_token(
                    &token,
                    &[TT::SEMICOLON],
                ));
                self.synchronise();
            }
        }

        if !errors.is_empty() {
            return Err(errors);
        }
        Ok(statments)
    }

    fn declaration(&mut self) -> Result<Statment<'a>, ParserError> {
        let token = self.next().ok_or(ParserError::UnexpectedEOF)?;
        let name = match token.token_value {
            Some(TV::Identifier(name)) => Ok(name),
            _ => Err(ParserError::unexpected_token(&token, &[TT::IDENTIFIER])),
        }?;

        if let Some(next_token) = self.peek()
            && next_token.kind == TT::EQUAL
        {
            self.next();
            let value = self.expression(0)?;
            Ok(Statment::Declaration {
                name,
                expression: Some(value),
            })
        } else {
            Ok(Statment::Declaration {
                name,
                expression: None,
            })
        }
    }

    fn expression(
        &mut self,
        current_precedence: usize,
    ) -> Result<Expr<'a>, ParserError> {
        let mut lhs = self.parse_prefix()?;

        while let Some(infix) = self
            .peek()
            .and_then(|token| BinaryOp::try_from(token.kind).ok())
        {
            let (l_precedence, r_precedence) = infix_precedence(infix);
            if l_precedence < current_precedence {
                break;
            }
            let token = self.next().expect("Retriving a peeked value");
            let rhs = self.expression(r_precedence).wrap_err_with(|| {
                format!("Failed reading rhs for {token:?}")
            })?;
            lhs = Expr::new_binary(
                Box::new(lhs),
                infix,
                Box::new(rhs),
                token.line,
            );
        }
        Ok(lhs)
    }

    pub fn parse_prefix(&mut self) -> Result<Expr<'a>, ParserError> {
        let token = self.next().ok_or(ParserError::UnexpectedEOF)?;

        if let Some(token_value) = token.token_value {
            return Ok(build_value(token_value, token.line));
        }

        if let Ok(unary_op) = UnaryOperator::try_from(token.kind) {
            return self.build_unary(unary_op, token.line);
        }

        if token.kind == TT::LEFT_PAREN {
            return self.build_group();
        }

        Err(ParserError::unexpected_token(
            &token,
            &[TT::BANG, TT::MINUS, TT::LEFT_PAREN],
        ))
    }

    fn build_unary(
        &mut self,
        operator: UnaryOperator,
        line: usize,
    ) -> Result<Expr<'a>, ParserError> {
        let precedence = prefix_precedence(operator);
        let expr = Box::new(self.expression(precedence)?);
        Ok(Expr::new_unary(operator, expr, line))
    }

    fn build_group(&mut self) -> Result<Expr<'a>, ParserError> {
        let inner = self.expression(0)?;
        let token = self
            .next()
            .ok_or_else(|| ParserError::expected_token(&[TT::RIGHT_PAREN]))?;

        match token.kind {
            TT::RIGHT_PAREN => {
                Ok(Expr::new_grouping(Box::new(inner), token.line))
            }
            _ => Err(ParserError::unexpected_token(&token, &[TT::RIGHT_PAREN])),
        }
    }

    fn synchronise(&mut self) {
        while let Some(token) = self.next() {
            if token.kind == TT::SEMICOLON {
                break;
            }
            if self.peek().is_some_and(|next_token| {
                [
                    TT::CLASS,
                    TT::FUN,
                    TT::VAR,
                    TT::FOR,
                    TT::WHILE,
                    TT::PRINT,
                    TT::RETURN,
                ]
                .contains(&next_token.kind)
            }) {
                break;
            }
        }
    }
}

fn build_value(value: TV, line: usize) -> Expr {
    match value {
        TV::String(text) => {
            Expr::new_literal(Value::String(text.to_owned()), line)
        }
        TV::Number(number) => Expr::new_literal(Value::Number(number), line),
        TV::False => Expr::new_literal(Value::Boolean(false), line),
        TV::True => Expr::new_literal(Value::Boolean(true), line),
        TV::Nil => Expr::new_literal(Value::Nil, line),
        TV::Identifier(name) => Expr::new_identifier(name, line),
        TV::Comment(..) => {
            unreachable!("Shouldn't be emitted")
        }
    }
}
