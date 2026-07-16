pub mod parser_error;
use crate::error_utils::StageError;
use crate::expressions::BinaryOperator as BinaryOp;
use crate::expressions::ExprKind;
use crate::expressions::LogicalOperator;
use crate::expressions::{Expr, Statement, UnaryOperator, Value};
use crate::parser::parser_error::{ParserError, WrapErr};
use crate::token::Token;
use crate::token::TokenValue as TV;
use crate::token_type::TokenType as TT;
use log::trace;
use std::iter::Peekable;
use std::vec::IntoIter;
use std::vec::Vec;

pub const fn prefix_precedence(token_type: UnaryOperator) -> usize {
    match token_type {
        UnaryOperator::BANG | UnaryOperator::MINUS => 14,
    }
}

pub const fn logical_precedence(token_type: LogicalOperator) -> usize {
    match token_type {
        LogicalOperator::AND => 1,
        LogicalOperator::OR => 2,
    }
}

pub const fn infix_precedence(token_type: BinaryOp) -> (usize, usize) {
    match token_type {
        BinaryOp::COMMA => (2, 3),
        BinaryOp::QUESTION_MARK | BinaryOp::COLON => (5, 4),
        BinaryOp::EQUAL_EQUAL | BinaryOp::BANG_EQUAL => (6, 7),
        BinaryOp::LESS
        | BinaryOp::LESS_EQUAL
        | BinaryOp::GREATER
        | BinaryOp::GREATER_EQUAL => (8, 9),
        BinaryOp::PLUS | BinaryOp::MINUS => (10, 11),
        BinaryOp::STAR | BinaryOp::SLASH => (12, 13),
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
    loop_depth: usize,
}

pub fn parse(
    tokens: Vec<Token>,
) -> Result<Vec<Statement>, Vec<impl Into<StageError> + use<>>> {
    Parser::new(tokens).parse()
}

impl<'a> Parser<'a> {
    pub fn new(tokens: Vec<Token<'a>>) -> Self {
        Parser {
            tokens: TokenIter(tokens.into_iter()).peekable(),
            loop_depth: 0,
        }
    }

    fn consume_if(
        &mut self,
        token_types: &'static [TT],
    ) -> Result<Token<'a>, ParserError> {
        if let Some(token) = self.tokens.peek() {
            if token_types.contains(&token.kind) {
                Ok(self.tokens.next().expect("Unwrapping peeked value"))
            } else {
                Err(ParserError::unexpected_token(token, token_types))
            }
        } else {
            Err(ParserError::expected_token(token_types))
        }
    }

    fn consume_name(&mut self) -> Result<&'a str, ParserError> {
        trace!("Consume name");
        let token = self.consume_if(&[TT::IDENTIFIER])?;
        let Some(TV::Identifier(name)) = token.token_value else {
            return Err(ParserError::unexpected_token(
                &token,
                &[TT::IDENTIFIER],
            ));
        };
        Ok(name)
    }

    fn parse(&mut self) -> Result<Vec<Statement<'a>>, Vec<ParserError>> {
        let mut statements = Vec::new();
        let mut errors = Vec::new();
        while self.tokens.peek().is_some() {
            match self.statement() {
                Ok(statement) => statements.push(statement),
                Err(e) => errors.extend(e),
            }
        }

        if !errors.is_empty() {
            return Err(errors);
        }
        Ok(statements)
    }

    fn statement(&mut self) -> Result<Statement<'a>, Vec<ParserError>> {
        match self.tokens.peek() {
            None => Err(vec![ParserError::UnexpectedEOF]),
            Some(token) if token.kind == TT::LEFT_BRACE => {
                self.tokens.next();
                self.block()
            }
            Some(token) if token.kind == TT::FUN => {
                let line =
                    self.tokens.next().expect("Unwrapping a peeked value").line;
                self.function(line)
            }
            _ => self.keyword(),
        }
    }

    fn block(&mut self) -> Result<Statement<'a>, Vec<ParserError>> {
        trace!("Block");
        let mut members = Vec::new();
        let mut errors = Vec::new();
        while let Some(next_token) = self.tokens.peek()
            && next_token.kind != TT::RIGHT_BRACE
        {
            let statement = if next_token.kind == TT::LEFT_BRACE {
                self.block()
                    .wrap_err_with(|| "Failed generating block".to_owned())
            } else {
                self.keyword()
                    .wrap_err_with(|| "Failed generating line".to_owned())
            };
            match statement {
                Ok(s) => members.push(s),
                Err(e) => {
                    self.synchronise();
                    errors.push(e);
                }
            }
        }
        if let Err(e) = self.consume_if(&[TT::RIGHT_BRACE]) {
            errors.push(e);
        }

        if !errors.is_empty() {
            return Err(errors);
        }
        Ok(Statement::Group(members))
    }

    fn function(
        &mut self,
        line: usize,
    ) -> Result<Statement<'a>, Vec<ParserError>> {
        trace!("Function");
        let name = self.consume_name()?;
        self.consume_if(&[TT::LEFT_PAREN])?;

        let mut params = vec![];
        while let Ok(param) = self.consume_name() {
            params.push(param);
            if params.len() >= 255 {
                return Err(vec![ParserError::TooManyArguments { line }]);
            }
            if self.consume_if(&[TT::COMMA]).is_err() {
                break;
            }
        }

        self.consume_if(&[TT::RIGHT_PAREN])?;
        self.consume_if(&[TT::LEFT_BRACE])?;
        let body = Box::new(self.block()?);

        Ok(Statement::Declaration {
            name,
            expression: Some(Expr::function(name, params, body, line)),
        })
    }

    fn keyword(&mut self) -> Result<Statement<'a>, Vec<ParserError>> {
        trace!("Keyword");
        let next_token =
            self.tokens.peek().ok_or(ParserError::UnexpectedEOF)?;

        let statement = match next_token.kind {
            TT::VAR => {
                self.tokens.next();
                let declaration = self.declaration()?;
                if self.tokens.peek().is_some() {
                    self.consume_if(&[TT::SEMICOLON])?;
                }
                declaration
            }
            TT::PRINT => {
                trace!("Print");
                self.tokens.next();
                let statement = self.expression(0).map(Statement::Print)?;
                if self.tokens.peek().is_some() {
                    self.consume_if(&[TT::SEMICOLON])?;
                }
                statement
            }
            TT::IF => {
                self.tokens.next();
                self.if_statement()?
            }
            TT::WHILE => {
                self.tokens.next();
                self.while_statement()?
            }
            TT::FOR => {
                self.tokens.next();
                self.for_statement()?
            }
            TT::BREAK => {
                if self.loop_depth != 0 {
                    return Err(vec![ParserError::unexpected_token(
                        &self.tokens.next().expect("Unwrapping a peeked value"),
                        &[],
                    )]);
                }
                if self.tokens.peek().is_some() {
                    self.consume_if(&[TT::SEMICOLON])?;
                }
                Statement::Break
            }
            TT::RETURN => {
                let line =
                    self.tokens.next().expect("Unwrapping a peeked value").line;
                self.return_statement(line)?
            }
            _ => self.expression(0).map(Statement::Expression)?,
        };
        Ok(statement)
    }

    fn declaration(&mut self) -> Result<Statement<'a>, ParserError> {
        trace!("Declaration");
        let name = self.consume_name()?;
        let expression =
            if self.tokens.peek().is_some_and(|t| t.kind == TT::EQUAL) {
                self.tokens.next();
                Some(self.expression(0)?)
            } else {
                None
            };
        Ok(Statement::Declaration { name, expression })
    }

    fn if_statement(&mut self) -> Result<Statement<'a>, Vec<ParserError>> {
        trace!("If");
        self.consume_if(&[TT::LEFT_PAREN])?;
        let condition = self.expression(0)?;
        self.consume_if(&[TT::RIGHT_PAREN])?;

        let true_branch = Box::new(self.statement()?);
        let false_branch = match self.consume_if(&[TT::ELSE]) {
            Ok(_) => Some(Box::new(self.statement()?)),
            Err(_) => None,
        };

        Ok(Statement::If {
            condition,
            true_branch,
            false_branch,
        })
    }

    fn while_statement(&mut self) -> Result<Statement<'a>, Vec<ParserError>> {
        self.consume_if(&[TT::LEFT_PAREN])?;
        let condition = self.expression(0)?;
        self.consume_if(&[TT::RIGHT_PAREN])?;

        self.loop_depth += 1;
        let body = Box::new(self.statement()?);
        self.loop_depth -= 1;

        Ok(Statement::While { condition, body })
    }

    fn for_statement(&mut self) -> Result<Statement<'a>, Vec<ParserError>> {
        trace!("For");
        self.consume_if(&[TT::LEFT_PAREN])?;

        let mut next_token =
            self.tokens.peek().ok_or(ParserError::UnexpectedEOF)?;
        let line = next_token.line;

        let initialiser = match next_token.kind {
            TT::SEMICOLON => None,
            TT::VAR => {
                self.tokens.next();
                Some(self.declaration()?)
            }
            _ => Some(self.expression(0).map(Statement::Expression)?),
        };
        self.consume_if(&[TT::SEMICOLON])?;
        trace!("For condition");

        next_token = self.tokens.peek().ok_or(ParserError::UnexpectedEOF)?;
        let condition = match next_token.kind {
            TT::SEMICOLON => None,
            _ => Some(self.expression(0)?),
        };
        self.consume_if(&[TT::SEMICOLON])?;
        trace!("For increment");

        next_token = self.tokens.peek().ok_or(ParserError::UnexpectedEOF)?;
        let increment = match next_token.kind {
            TT::SEMICOLON => None,
            _ => Some(self.expression(0)?),
        };
        self.consume_if(&[TT::RIGHT_PAREN])?;
        trace!("For body");

        let body = Box::new(self.statement()?);

        Ok(Statement::r#for(
            initialiser,
            condition,
            increment,
            body,
            line,
        ))
    }

    fn return_statement(
        &mut self,
        line: usize,
    ) -> Result<Statement<'a>, ParserError> {
        let return_expr = if matches!(
            self.tokens.peek().ok_or(ParserError::UnexpectedEOF)?.kind,
            TT::SEMICOLON
        ) {
            None
        } else {
            Some(self.expression(0)?)
        };

        self.consume_if(&[TT::SEMICOLON])?;

        Ok(Statement::Return {
            line,
            value: return_expr,
        })
    }

    fn expression(
        &mut self,
        current_precedence: usize,
    ) -> Result<Expr<'a>, ParserError> {
        trace!("Expression");
        let lhs = self.build_logical(current_precedence)?;

        if let Some(infix) = self.tokens.peek()
            && infix.kind == TT::EQUAL
        {
            let token = self.tokens.next().expect("Retriving a peeked value");
            let rhs = self.expression(current_precedence)?;

            if let ExprKind::Identifier(name) = lhs.kind {
                return Ok(Expr::assignment(name, Box::new(rhs), token.line));
            }
            return Err(ParserError::InvalidAssignmentTarget {
                line: token.line,
            });
        }

        Ok(lhs)
    }

    pub fn build_logical(
        &mut self,
        current_precedence: usize,
    ) -> Result<Expr<'a>, ParserError> {
        trace!("Logical");
        let mut lhs = self.build_binary(current_precedence)?;

        while let Some(infix) = self
            .tokens
            .peek()
            .and_then(|token| LogicalOperator::try_from(token.kind).ok())
        {
            let precedence = logical_precedence(infix);
            if precedence < current_precedence {
                break;
            }
            let token = self.tokens.next().expect("Retriving a peeked value");
            let rhs = self.build_logical(precedence).wrap_err_with(|| {
                format!("Failed reading rhs for {token:?}")
            })?;
            lhs =
                Expr::logical(Box::new(lhs), infix, Box::new(rhs), token.line);
        }
        Ok(lhs)
    }

    pub fn build_binary(
        &mut self,
        current_precedence: usize,
    ) -> Result<Expr<'a>, ParserError> {
        trace!("Binary");
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
            lhs = Expr::binary(Box::new(lhs), infix, Box::new(rhs), token.line);
        }
        Ok(lhs)
    }

    pub fn parse_prefix(&mut self) -> Result<Expr<'a>, ParserError> {
        trace!("Prefix");
        let token = self.tokens.next().ok_or(ParserError::UnexpectedEOF)?;

        if let Some(token_value) = token.token_value {
            let mut expr = build_value(token_value, token.line);

            loop {
                if let Some(token) = self.tokens.peek()
                    && matches!(token.kind, TT::LEFT_PAREN)
                {
                    self.tokens.next();
                    expr = self.build_call(expr)?;
                } else {
                    break;
                }
            }
            return Ok(expr);
        }

        if let Ok(unary_op) = UnaryOperator::try_from(token.kind) {
            return self.build_unary(unary_op, token.line);
        }

        if token.kind == TT::LEFT_PAREN {
            return self.build_group();
        }

        // todo!(
        //    "Error message dose not reflect that this accepts any valid value"
        //);
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
        Ok(Expr::unary(operator, expr, line))
    }

    fn build_group(&mut self) -> Result<Expr<'a>, ParserError> {
        let inner = self.expression(0)?;
        let token = self.consume_if(&[TT::RIGHT_PAREN])?;
        Ok(Expr::grouping(Box::new(inner), token.line))
    }

    fn build_call(
        &mut self,
        callee: Expr<'a>,
    ) -> Result<Expr<'a>, ParserError> {
        trace!("Call");
        let mut arguments = Vec::new();

        loop {
            arguments.push(self.expression(5)?);
            if arguments.len() >= 255 {
                return Err(ParserError::TooManyArguments {
                    line: arguments[arguments.len() - 1].line,
                });
                todo!(
                    "This needs to report the error, but not go into panic mode???"
                );
            }
            if self.consume_if(&[TT::COMMA]).is_err() {
                break;
            }
        }

        self.consume_if(&[TT::RIGHT_PAREN])?;

        Ok(Expr::call(Box::new(callee), arguments))
    }

    fn synchronise(&mut self) {
        trace!("Synchronising");
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
        TV::String(text) => Expr::literal(Value::String(text.to_owned()), line),
        TV::Number(number) => Expr::literal(Value::Number(number), line),
        TV::False => Expr::literal(Value::Boolean(false), line),
        TV::True => Expr::literal(Value::Boolean(true), line),
        TV::Nil => Expr::literal(Value::Nil, line),
        TV::Identifier(name) => Expr::identifier(name, line),
        TV::Comment(..) => {
            unreachable!("Shouldn't be emitted")
        }
    }
}
