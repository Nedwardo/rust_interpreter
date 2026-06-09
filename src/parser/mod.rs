pub mod parser_error;
use crate::error_utils::StageError;
use crate::expressions::BinaryOperator as BinaryOp;
use crate::expressions::ExprKind;
use crate::expressions::{Expr, Statement, UnaryOperator, Value};
use crate::parser::parser_error::{ParserError, WrapErr};
use crate::token::Token;
use crate::token::TokenValue as TV;
use crate::token_type::TokenType as TT;
use std::iter::Peekable;
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

struct TokenIter<'a>(IntoIter<Token<'a>>);
impl<'a> Iterator for TokenIter<'a> {
    type Item = Token<'a>;
    fn next(&mut self) -> Option<Token<'a>> {
        self.0.by_ref().find(|t| t.kind != TT::COMMENT)
    }
}

pub struct Parser<'a> {
    tokens: Peekable<TokenIter<'a>>,
}

pub fn parse(
    tokens: Vec<Token>,
) -> Result<Vec<Statement>, Vec<impl Into<StageError>>> {
    Parser::new(tokens).parse()
}

impl<'a> Parser<'a> {
    pub fn new(tokens: Vec<Token<'a>>) -> Self {
        Parser {
            tokens: TokenIter(tokens.into_iter()).peekable(),
        }
    }

    fn parse(&mut self) -> Result<Vec<Statement<'a>>, Vec<ParserError>> {
        let mut statments = Vec::new();
        let mut errors = Vec::new();
        while let Some(token) = self.tokens.peek() {
            if token.kind == TT::LEFT_BRACE {
                self.tokens.next();
                match self.block() {
                    Ok(statment) => statments.push(statment),
                    Err(e) => errors.extend(e),
                }
            } else {
                match self.statment() {
                    Ok(statment) => statments.push(statment),
                    Err(e) => errors.push(e),
                }
            }
        }

        if !errors.is_empty() {
            return Err(errors);
        }
        Ok(statments)
    }

    fn block(&mut self) -> Result<Statement<'a>, Vec<ParserError>> {
        let mut members = Vec::new();
        let mut errors = Vec::new();
        while let Some(next_token) = self.tokens.peek()
            && next_token.kind != TT::RIGHT_BRACE
        {
            let statment = if next_token.kind == TT::LEFT_BRACE {
                self.block()
                    .wrap_err_with(|| "Failed generating block".to_owned())
            } else {
                self.statment()
                    .wrap_err_with(|| "Failed generating block".to_owned())
            };
            match statment {
                Ok(s) => members.push(s),
                Err(e) => {
                    self.synchronise();
                    errors.push(e);
                }
            }
        }
        if let Some(token) = self.tokens.next() {
            if token.kind != TT::RIGHT_BRACE {
                errors.push(ParserError::unexpected_token(
                    &token,
                    &[TT::RIGHT_BRACE],
                ));
            }
        } else {
            errors.push(ParserError::expected_token(&[TT::RIGHT_BRACE]));
        }

        if !errors.is_empty() {
            return Err(errors);
        }
        Ok(Statement::Group(members))
    }

    fn statment(&mut self) -> Result<Statement<'a>, ParserError> {
        let next_token =
            self.tokens.peek().ok_or(ParserError::UnexpectedEOF)?;
        let statment = if next_token.kind == TT::VAR {
            self.tokens.next();
            self.declaration()?
        } else if next_token.kind == TT::PRINT {
            self.tokens.next();
            self.expression(0).map(Statement::Print)?
        } else {
            self.expression(0).map(Statement::Expression)?
        };
        if let Some(token) = self.tokens.peek()
            && token.kind != TT::SEMICOLON
        {
            Err(ParserError::unexpected_token(token, &[TT::SEMICOLON]))
        } else {
            self.tokens.next();
            Ok(statment)
        }
    }

    fn declaration(&mut self) -> Result<Statement<'a>, ParserError> {
        let token =
            self.tokens.next().ok_or(ParserError::EOFWhileExpecting {
                expected_token_types: &[TT::IDENTIFIER],
            })?;

        let Some(TV::Identifier(name)) = token.token_value else {
            return Err(ParserError::unexpected_token(
                &token,
                &[TT::IDENTIFIER],
            ));
        };

        let expression =
            if self.tokens.peek().is_some_and(|t| t.kind == TT::EQUAL) {
                self.tokens.next();
                Some(self.expression(0)?)
            } else {
                None
            };
        Ok(Statement::Declaration { name, expression })
    }

    fn expression(
        &mut self,
        current_precedence: usize,
    ) -> Result<Expr<'a>, ParserError> {
        let lhs = self.build_binary(current_precedence)?;

        if let Some(infix) = self.tokens.peek()
            && infix.kind == TT::EQUAL
        {
            let token = self.tokens.next().expect("Retriving a peeked value");
            let rhs = self.expression(current_precedence)?;

            if let ExprKind::Identifier(name) = lhs.kind {
                return Ok(Expr::new_assignment(
                    name,
                    Box::new(rhs),
                    token.line,
                ));
            }
            return Err(ParserError::InvalidAssignmentTarget {
                line: token.line,
            });
        }

        Ok(lhs)
    }

    pub fn build_binary(
        &mut self,
        current_precedence: usize,
    ) -> Result<Expr<'a>, ParserError> {
        let mut lhs = self.parse_prefix()?;

        while let Some(infix) = self
            .tokens
            .peek()
            .and_then(|token| BinaryOp::try_from(token.kind).ok())
        {
            let (l_precedence, r_precedence) = infix_precedence(infix);
            if l_precedence < current_precedence {
                break;
            }
            let token = self.tokens.next().expect("Retriving a peeked value");
            let rhs = self.build_binary(r_precedence).wrap_err_with(|| {
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
        let token = self.tokens.next().ok_or(ParserError::UnexpectedEOF)?;

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
            .tokens
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
        while let Some(token) = self.tokens.next() {
            if token.kind == TT::SEMICOLON {
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
