pub mod parser_error;
use crate::expressions::Expr;
use crate::expressions::UnaryOperator;
use crate::expressions::Value;
use crate::parser::parser_error::ParserError;
use crate::parser::parser_error::WrapErr;
use crate::token::Token;
use crate::token::TokenValue as VT;
use crate::token_type::TokenType as TT;
use std::iter::Peekable;
use std::vec::IntoIter;
use std::vec::Vec;

pub const fn prefix_precedence(token_type: UnaryOperator) -> usize {
    match token_type {
        UnaryOperator::BANG | UnaryOperator::MINUS => 13,
    }
}

pub const fn infix_precedence(token_type: TT) -> Option<(usize, usize)> {
    match token_type {
        TT::COMMA => Some((1, 2)),
        TT::QUESTION_MARK | TT::COLON => Some((4, 3)),
        TT::EQUAL_EQUAL | TT::BANG_EQUAL => Some((5, 6)),
        TT::LESS | TT::LESS_EQUAL | TT::GREATER | TT::GREATER_EQUAL => {
            Some((7, 8))
        }
        TT::PLUS | TT::MINUS => Some((9, 10)),
        TT::STAR | TT::SLASH => Some((11, 12)),
        _ => None,
    }
}

pub struct Parser<'a> {
    tokens: IntoIter<Token<'a>>,
    peeked: Option<Token<'a>>,
}

pub fn parse(tokens: Vec<Token<'_>>) -> Result<Expr<'_>, ParserError<'_>> {
    Parser::new(tokens).parse(0)
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
        self.tokens
            .by_ref()
            .find(|token| token.token_type != TT::COMMENT)
    }

    fn peek(&mut self) -> Option<&Token<'a>> {
        self.peeked = self.next();
        self.peeked.as_ref()
    }

    fn parse(
        &mut self,
        current_precedence: usize,
    ) -> Result<Expr<'a>, ParserError<'a>> {
        let mut lhs = self
            .parse_prefix()
            .wrap_err("Failed reading prefix".to_owned())?;

        while let Some(token) = self.peek() {
            let token_type = token.token_type;

            let Some((l_precedence, r_precedence)) =
                infix_precedence(token_type)
            else {
                break;
            };

            if l_precedence < current_precedence {
                break;
            }
            self.next();
            let rhs = self.parse(r_precedence).wrap_err(
                format!("Failed reading rhs for {token_type:?}").to_owned(),
            )?;
            lhs = Expr::Binary {
                left: Box::new(lhs),
                operator: token_type,
                right: Box::new(rhs),
            };
        }
        Ok(lhs)
    }

    pub fn parse_prefix(&mut self) -> Result<Expr<'a>, ParserError<'a>> {
        let token = self.next().ok_or(ParserError::UnexpectedEOF)?;

        if let Some(token_value) = token.token_value {
            return Ok(match token_value {
                VT::String(text) => {
                    Expr::Literal(Value::String(text.to_owned()))
                }
                VT::Number(number) => Expr::Literal(Value::Number(number)),
                VT::False => Expr::Literal(Value::Boolean(false)),
                VT::True => Expr::Literal(Value::Boolean(true)),
                VT::Nil => Expr::Literal(Value::Nil),
                VT::Identifier(name) => Expr::Identifier(name),
                VT::Comment(..) => {
                    unreachable!("Shouldn't be emitted")
                }
            });
        }

        match token.token_type {
            TT::BANG => self.build_unary(UnaryOperator::BANG),
            TT::MINUS => self.build_unary(UnaryOperator::MINUS),
            TT::LEFT_PAREN => self.build_group(),
            _ => Err(ParserError::UnexpectedToken {
                token: Some(token),
                expected_token_types: &[TT::BANG, TT::MINUS, TT::LEFT_PAREN],
            }),
        }
    }

    fn build_unary(
        &mut self,
        unary_op: UnaryOperator,
    ) -> Result<Expr<'a>, ParserError<'a>> {
        let precedence = prefix_precedence(unary_op);
        let expr = Box::new(self.parse(precedence)?);
        Ok(Expr::Unary {
            operator: unary_op,
            expr,
        })
    }

    fn build_group(&mut self) -> Result<Expr<'a>, ParserError<'a>> {
        let inner = self.parse(0)?;
        let token = self.next().ok_or(ParserError::UnexpectedToken {
            token: None,
            expected_token_types: &[TT::RIGHT_PAREN],
        })?;

        match token.token_type {
            TT::RIGHT_PAREN => Ok(Expr::Grouping(Box::new(inner))),
            _ => Err(ParserError::UnexpectedToken {
                token: Some(token),
                expected_token_types: &[TT::RIGHT_PAREN],
            }),
        }
    }

    fn syncronise(tokens: &mut Peekable<IntoIter<Token<'_>>>) {
        while let Some(token) = tokens.next() {
            if token.token_type == TT::SEMICOLON {
                break;
            }
            if tokens.peek().is_some_and(|next_token| {
                [
                    TT::CLASS,
                    TT::FUN,
                    TT::VAR,
                    TT::FOR,
                    TT::WHILE,
                    TT::PRINT,
                    TT::RETURN,
                ]
                .contains(&next_token.token_type)
            }) {
                break;
            }
        }
    }
}
