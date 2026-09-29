pub mod parser_error;
use crate::error_utils::StageError;
use crate::evaluator::environment::VariableBinding;
use crate::expressions::BinaryOperator as BinaryOp;
use crate::expressions::ExprKind;
use crate::expressions::FunctionKind;
use crate::expressions::LogicalOperator as LogicalOp;
use crate::expressions::{Expr, Function, Statement, Value};
use crate::operator_subset;
use crate::parser::parser_error::ParserError as Error;
use crate::token::Span;
use crate::token::Token;
use crate::token_type::KeywordToken as KT;
use crate::token_type::LiteralToken as LT;
use crate::token_type::TokenType as TT;
use crate::token_type::TokenTypeSubset;
use log::debug;
use std::iter::Peekable;
use std::vec::IntoIter;
use std::vec::Vec;

pub const fn logical_precedence(token_type: LogicalOp) -> usize {
    match token_type {
        LogicalOp::And => 1,
        LogicalOp::Or => 2,
    }
}

pub const fn infix_precedence(token_type: BinaryOp) -> (usize, usize) {
    match token_type {
        BinaryOp::Comma => (2, 3),
        BinaryOp::QuestionMark | BinaryOp::Colon => (5, 4),
        BinaryOp::EqualEqual | BinaryOp::BangEqual => (6, 7),
        BinaryOp::Less
        | BinaryOp::LessEqual
        | BinaryOp::Greater
        | BinaryOp::GreaterEqual => (8, 9),
        BinaryOp::Plus | BinaryOp::Minus => (10, 11),
        BinaryOp::Star | BinaryOp::Slash => (12, 13),
    }
}

operator_subset!(Keyword, {Var, Fun, Print, If, While, For, Break, Return} );

#[derive(Clone)]
struct TokenCursor {
    tokens: Peekable<IntoIter<Token>>,
    checked_tokens: Vec<TT>,
}
impl TokenCursor {
    fn new(tokens: IntoIter<Token>) -> Self {
        Self {
            tokens: tokens.peekable(),
            checked_tokens: vec![],
        }
    }

    fn next(&mut self) -> Option<Token> {
        self.checked_tokens.clear();
        self.tokens.next()
    }

    fn peek(&mut self) -> Option<&Token> {
        self.tokens.peek()
    }

    fn consume_if(&mut self, token_types: &[TT]) -> Result<Token, Error> {
        self.checked_tokens.extend(token_types);
        match self.tokens.peek() {
            Some(t) if token_types.contains(&t.token_kind) => {
                self.checked_tokens.clear();
                Ok(self.next().expect("Unwrapping peeked value"))
            }
            Some(t) => Err(Error::unexpected_token(t, &self.checked_tokens)),
            None => Err(Error::expected_token(&self.checked_tokens)),
        }
    }

    fn consume_identifier_span(&mut self) -> Result<Span, Error> {
        debug!("Consume name");
        let token = self.consume_if(&[TT::Literal(LT::Identifier)])?;
        Ok(token.span)
    }

    #[allow(clippy::map_err_ignore, reason = "Error data is present in token")]
    fn peek_token_subset<Op: TokenTypeSubset>(
        &mut self,
    ) -> Result<(Op, Span), Error> {
        self.checked_tokens.extend(Op::VARIANTS);
        let token = self
            .tokens
            .peek()
            .ok_or_else(|| Error::expected_token(&self.checked_tokens))?;
        Op::try_from(token.token_kind)
            .map(|op| (op, token.span))
            .map_err(|_| Error::unexpected_token(token, &self.checked_tokens))
    }

    fn consume_semicolon_or_eof(&mut self) -> Result<(), Error> {
        if self.peek().is_some() {
            self.consume_if(&[TT::Keyword(KT::Semicolon)])?;
        }
        Ok(())
    }

    fn consume_if_map_success<T>(
        &mut self,
        expected: &[TT],
        f: impl FnOnce(&Token) -> Result<T, Error>,
    ) -> Result<T, Error> {
        self.checked_tokens.extend(expected);

        let token = self
            .tokens
            .peek()
            .ok_or_else(|| Error::expected_token(&self.checked_tokens))?;

        let out = f(token)?;

        self.next();
        self.checked_tokens.clear();
        Ok(out)
    }

    fn unexpected(&self, token: &Token) -> Error {
        Error::unexpected_token(token, &self.checked_tokens)
    }

    fn failed_to_match(&mut self) -> Error {
        match self.tokens.peek() {
            Some(token) => Error::unexpected_token(token, &self.checked_tokens),
            None => Error::expected_token(&self.checked_tokens),
        }
    }
}

pub struct Parser<'a> {
    source: &'a str,
    tokens: TokenCursor,
    loop_depth: usize,
}

pub fn parse(
    source: &str,
    tokens: Vec<Token>,
) -> Result<Vec<Statement<'_>>, StageError> {
    Ok(Parser::new(source, tokens).parse()?)
}

impl<'a> Parser<'a> {
    pub fn new(source: &'a str, tokens: Vec<Token>) -> Self {
        Parser {
            source,
            tokens: TokenCursor::new(tokens.into_iter()),
            loop_depth: 0,
        }
    }

    fn parse(&mut self) -> Result<Vec<Statement<'a>>, Error> {
        let mut statements = Vec::new();
        let mut errors = Vec::new();
        while self.tokens.peek().is_some() {
            match self.statement() {
                Ok(statement) => statements.push(statement),
                Err(e) => errors.push(e),
            }
        }

        if !errors.is_empty() {
            return Err(Error::block_error(errors));
        }
        Ok(statements)
    }

    fn statement(&mut self) -> Result<Statement<'a>, Error> {
        if self.tokens.peek().is_none() {
            return Err(Error::unexpected_eof("Start of statement"));
        }
        if self
            .tokens
            .consume_if(&[TT::Keyword(KT::LeftBrace)])
            .is_ok()
        {
            self.block()
        } else {
            self.keyword()
        }
    }

    fn block(&mut self) -> Result<Statement<'a>, Error> {
        debug!("Block");
        let mut members = Vec::new();
        let mut errors = Vec::new();
        while let Some(next_token) = self.tokens.peek()
            && next_token.token_kind != KT::RightBrace
        {
            let statement = self.statement();
            match statement {
                Ok(s) => members.push(s),
                Err(err) => {
                    if err.synchronise {
                        self.synchronise();
                    }
                    errors.push(err);
                }
            }
        }

        if let Err(e) = self.tokens.consume_if(&[TT::Keyword(KT::RightBrace)]) {
            errors.push(e);
        }

        if !errors.is_empty() {
            return Err(Error::block_error(errors));
        }
        Ok(Statement::Group(members))
    }

    fn function_declaration(
        &mut self,
        span: Span,
    ) -> Result<Statement<'a>, Error> {
        debug!("Function");
        let is_named = matches!(self.tokens.peek(), Some(t) if t.token_kind == LT::Identifier);
        if is_named {
            let span = self.tokens.consume_identifier_span()?;
            let binding = VariableBinding::from_span(span, self.source);
            Ok(Statement::FunctionDeclaration(self.function(binding)?))
        } else {
            Ok(Statement::Expression(Expr::lambda(self.function(
                VariableBinding {
                    span,
                    name: "lambda",
                },
            )?)))
        }
    }

    fn function(
        &mut self,
        binding: VariableBinding<'a>,
    ) -> Result<Function<'a>, Error> {
        self.tokens.consume_if(&[TT::Keyword(KT::LeftParen)])?;

        let mut params = vec![];
        while let Ok(param) = self.tokens.consume_identifier_span() {
            params.push(VariableBinding::from_span(param, self.source));
            if params.len() >= 255 {
                return Err(Error::too_many_arguments(binding.span, true));
            }
            if self.tokens.consume_if(&[TT::Keyword(KT::Comma)]).is_err() {
                break;
            }
        }

        self.tokens.consume_if(&[TT::Keyword(KT::RightParen)])?;
        self.tokens.consume_if(&[TT::Keyword(KT::LeftBrace)])?;
        let body = Box::new(self.block()?);
        Ok(Function {
            binding,
            params,
            body: FunctionKind::Lox(body),
        })
    }

    fn keyword(&mut self) -> Result<Statement<'a>, Error> {
        debug!("Keyword");
        let Ok((keyword, span)) = self.tokens.peek_token_subset() else {
            // no keyword token, treat as an expression statement
            let statement = self.expression(0).map(Statement::Expression)?;
            self.tokens.consume_semicolon_or_eof()?;
            return Ok(statement);
        };
        let token = self.tokens.next().expect("Unwrapping a peeked value");

        let statement = match keyword {
            Keyword::Var => {
                let declaration = self.declaration()?;
                self.tokens.consume_semicolon_or_eof()?;
                declaration
            }
            Keyword::Fun => self.function_declaration(span)?,
            Keyword::Print => {
                debug!("Print");
                let statement = self.expression(0).map(Statement::Print)?;
                self.tokens.consume_semicolon_or_eof()?;
                statement
            }
            Keyword::If => self.if_statement()?,
            Keyword::While => self.while_statement()?,
            Keyword::For => self.for_statement()?,
            Keyword::Break => {
                if self.loop_depth != 0 {
                    return Err(self.tokens.unexpected(&token));
                }
                self.tokens.consume_semicolon_or_eof()?;
                Statement::Break
            }
            Keyword::Return => self.return_statement(span)?,
        };

        Ok(statement)
    }

    fn declaration(&mut self) -> Result<Statement<'a>, Error> {
        debug!("Declaration");
        let span = self.tokens.consume_identifier_span()?;
        let binding = VariableBinding::from_span(span, self.source);
        let expression =
            if self.tokens.consume_if(&[TT::Keyword(KT::Equal)]).is_ok() {
                Some(self.expression(0)?)
            } else {
                None
            };
        Ok(Statement::Declaration {
            binding,
            expression,
        })
    }

    fn if_statement(&mut self) -> Result<Statement<'a>, Error> {
        debug!("If");
        self.tokens.consume_if(&[TT::Keyword(KT::LeftParen)])?;
        let condition = self.expression(0)?;
        self.tokens.consume_if(&[TT::Keyword(KT::RightParen)])?;

        let true_branch = Box::new(self.statement()?);
        let false_branch =
            match self.tokens.consume_if(&[TT::Keyword(KT::Else)]) {
                Ok(_) => Some(Box::new(self.statement()?)),
                Err(_) => None,
            };

        Ok(Statement::If {
            condition,
            true_branch,
            false_branch,
        })
    }

    fn while_statement(&mut self) -> Result<Statement<'a>, Error> {
        self.tokens.consume_if(&[TT::Keyword(KT::LeftParen)])?;
        let condition = self.expression(0)?;
        self.tokens.consume_if(&[TT::Keyword(KT::RightParen)])?;

        self.loop_depth += 1;
        let body = Box::new(self.statement()?);
        self.loop_depth -= 1;

        Ok(Statement::While { condition, body })
    }

    fn for_statement(&mut self) -> Result<Statement<'a>, Error> {
        debug!("For");
        self.tokens.consume_if(&[TT::Keyword(KT::LeftParen)])?;

        let mut next_token = self
            .tokens
            .peek()
            .ok_or(Error::unexpected_eof("For statement"))?;
        let span = next_token.span;

        let initialiser = if let Some(token) = self.tokens.peek()
            && token.token_kind == KT::Semicolon
        {
            None
        } else if self.tokens.consume_if(&[TT::Keyword(KT::Var)]).is_ok() {
            Some(self.declaration()?)
        } else {
            Some(self.expression(0).map(Statement::Expression)?)
        };

        self.tokens.consume_if(&[TT::Keyword(KT::Semicolon)])?;
        debug!("For condition");

        next_token = self
            .tokens
            .peek()
            .ok_or(Error::unexpected_eof("For statement"))?;
        let condition = match next_token.token_kind {
            TT::Keyword(KT::Semicolon) => None,
            _ => Some(self.expression(0)?),
        };
        self.tokens.consume_if(&[TT::Keyword(KT::Semicolon)])?;
        debug!("For increment");

        next_token = self
            .tokens
            .peek()
            .ok_or(Error::unexpected_eof("For statement"))?;
        let increment = match next_token.token_kind {
            TT::Keyword(KT::Semicolon) => None,
            _ => Some(self.expression(0)?),
        };
        self.tokens.consume_if(&[TT::Keyword(KT::RightParen)])?;
        debug!("For body");

        let body = Box::new(self.statement()?);

        Ok(Statement::r#for(
            initialiser,
            condition,
            increment,
            body,
            span,
        ))
    }

    fn return_statement(&mut self, span: Span) -> Result<Statement<'a>, Error> {
        let return_expr = match self
            .tokens
            .peek()
            .ok_or(Error::unexpected_eof("Return statement"))?
            .token_kind
        {
            TT::Keyword(KT::Semicolon) => None,
            _ => Some(self.expression(0)?),
        };

        self.tokens.consume_if(&[TT::Keyword(KT::Semicolon)])?;

        Ok(Statement::Return {
            span,
            value: return_expr,
        })
    }

    fn expression(
        &mut self,
        current_precedence: usize,
    ) -> Result<Expr<'a>, Error> {
        debug!("Expression - {current_precedence}");
        let lhs = self.build_logical(current_precedence)?;

        if let Ok(token) = self.tokens.consume_if(&[TT::Keyword(KT::Equal)]) {
            debug!("Building an assignment from {token:?}");
            let rhs = self.expression(current_precedence)?;

            if let ExprKind::Identifier(name) = lhs.kind {
                return Ok(Expr::assignment(name, Box::new(rhs), token.span));
            }
            return Err(Error::invalid_assignment_target(token.span));
        }

        Ok(lhs)
    }

    pub fn build_logical(
        &mut self,
        current_precedence: usize,
    ) -> Result<Expr<'a>, Error> {
        debug!("Logical: {:?}", self.tokens.peek());
        let mut lhs = self.build_binary(current_precedence)?;

        while let Ok((infix, span)) = self.tokens.peek_token_subset() {
            let precedence = logical_precedence(infix);
            if precedence < current_precedence {
                break;
            }
            self.tokens.next();

            let rhs = self.build_logical(precedence)?;
            lhs = Expr::logical(Box::new(lhs), infix, Box::new(rhs), span);
        }
        Ok(lhs)
    }

    pub fn build_binary(
        &mut self,
        current_precedence: usize,
    ) -> Result<Expr<'a>, Error> {
        debug!("Binary");
        let mut lhs = self.parse_prefix()?;

        while let Ok((infix, span)) = self.tokens.peek_token_subset() {
            let (l_precedence, r_precedence) = infix_precedence(infix);
            if l_precedence < current_precedence {
                break;
            }
            self.tokens.next();

            let rhs = self.build_binary(r_precedence)?;
            lhs = Expr::binary(Box::new(lhs), infix, Box::new(rhs), span);
        }
        Ok(lhs)
    }

    pub fn parse_prefix(&mut self) -> Result<Expr<'a>, Error> {
        debug!("Prefix");

        if let Ok((unary_op, span)) = self.tokens.peek_token_subset() {
            self.tokens.next();
            let expr = Box::new(self.parse_prefix()?);
            return Ok(Expr::unary(unary_op, expr, span));
        }

        let mut expr = if let Ok(v) = self.parse_value() {
            v
        } else if self
            .tokens
            .consume_if(&[TT::Keyword(KT::LeftParen)])
            .is_ok()
        {
            let inner = self.expression(0)?;
            let token =
                self.tokens.consume_if(&[TT::Keyword(KT::RightParen)])?;
            Expr::grouping(Box::new(inner), token.span)
        } else if let Ok(token) =
            self.tokens.consume_if(&[TT::Keyword(KT::Fun)])
        {
            Expr::lambda(self.function(VariableBinding {
                span: token.span,
                name: "lambda",
            })?)
        } else {
            return Err(self.tokens.failed_to_match());
        };

        while self
            .tokens
            .consume_if(&[TT::Keyword(KT::LeftParen)])
            .is_ok()
        {
            expr = self.build_call(expr)?;
        }
        Ok(expr)
    }

    fn parse_value(&mut self) -> Result<Expr<'a>, Error> {
        self.tokens
            .consume_if_map_success(Value::token_types(), |token| {
                let value = Value::try_from(token, self.source)
                    .map_err(Error::value_error)?;
                Ok(Expr::literal(value, token.span))
            })
    }

    fn build_call(&mut self, callee: Expr<'a>) -> Result<Expr<'a>, Error> {
        debug!("Call");
        let mut arguments = Vec::new();

        while let Some(next) = self.tokens.peek()
            && next.token_kind != KT::RightParen
        {
            debug!("First call arg = {next:?}");
            arguments.push(self.expression(5)?);
            if arguments.len() >= 255 {
                return Err(Error::too_many_arguments(
                    arguments[arguments.len() - 1].span,
                    false,
                ));
            }
            if self.tokens.consume_if(&[TT::Keyword(KT::Comma)]).is_err() {
                break;
            }
        }

        self.tokens.consume_if(&[TT::Keyword(KT::RightParen)])?;

        Ok(Expr::call(Box::new(callee), arguments))
    }

    fn synchronise(&mut self) {
        debug!("Synchronising {:?}", self.tokens.peek());
        while let Some(token) = self.tokens.next() {
            if token.token_kind == KT::Semicolon {
                break;
            }
            if self.tokens.peek().is_some_and(|next_token| {
                [
                    TT::Keyword(KT::Class),
                    TT::Keyword(KT::Fun),
                    TT::Keyword(KT::Var),
                    TT::Keyword(KT::For),
                    TT::Keyword(KT::While),
                    TT::Keyword(KT::Print),
                    TT::Keyword(KT::Return),
                ]
                .contains(&next_token.token_kind)
            }) {
                break;
            }
        }
    }
}
