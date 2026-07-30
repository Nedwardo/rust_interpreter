pub mod parser_error;
use crate::error_utils::StageError;
use crate::expressions::BinaryOperator as BinaryOp;
use crate::expressions::ExprKind;
use crate::expressions::FunctionKind;
use crate::expressions::LogicalOperator as LogicalOp;
use crate::expressions::{Expr, Function, Statement, Value};
use crate::parser::parser_error::ParserError as Error;
use crate::token::Token;
use crate::token::TokenValue as TV;
use crate::token_type::OperatorSubset;
use crate::token_type::TokenType as TT;
use crate::token_type::ValueTokenTypes;
use crate::token_type::operator_subset;
use log::debug;
use std::iter::Peekable;
use std::vec::IntoIter;
use std::vec::Vec;

pub const fn logical_precedence(token_type: LogicalOp) -> usize {
    match token_type {
        LogicalOp::AND => 1,
        LogicalOp::OR => 2,
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

operator_subset!(Keyword, {VAR, FUN, PRINT, IF, WHILE, FOR, BREAK, RETURN} );

struct TokenIter<'a>(IntoIter<Token<'a>>);
impl<'a> Iterator for TokenIter<'a> {
    type Item = Token<'a>;
    fn next(&mut self) -> Option<Token<'a>> {
        self.0.by_ref().find(|t| t.kind != TT::COMMENT)
    }
}

struct TokenCursor<'a> {
    tokens: Peekable<TokenIter<'a>>,
    checked_tokens: Vec<TT>,
}
impl<'a> TokenCursor<'a> {
    fn new(tokens: TokenIter<'a>) -> Self {
        Self {
            tokens: tokens.peekable(),
            checked_tokens: vec![],
        }
    }

    fn next(&mut self) -> Option<Token<'a>> {
        self.checked_tokens.clear();
        self.tokens.next()
    }

    fn peek(&mut self) -> Option<&Token<'a>> {
        self.tokens.peek()
    }

    fn consume_if(
        &mut self,
        token_types: &'static [TT],
    ) -> Result<Token<'a>, Error> {
        self.checked_tokens.extend(token_types);
        match self.tokens.peek() {
            Some(t) if token_types.contains(&t.kind) => {
                Ok(self.next().expect("Unwrapping peeked value"))
            }
            Some(t) => Err(Error::unexpected_token(t, &self.checked_tokens)),
            None => Err(Error::expected_token(&self.checked_tokens)),
        }
    }

    fn consume_name(&mut self) -> Result<&'a str, Error> {
        debug!("Consume name");
        let token = self.consume_if(&[TT::IDENTIFIER])?;
        let Some(TV::Identifier(name)) = token.token_value else {
            unreachable!(
                "TT::Identifier's should all contain TV::Identifier's"
            );
        };
        Ok(name)
    }

    #[allow(clippy::map_err_ignore, reason = "Error data is present in token")]
    fn peek_token_subset<Op: OperatorSubset<TT>>(
        &mut self,
    ) -> Result<(Op, usize), Error> {
        self.checked_tokens.extend(Op::VARIANTS);
        let token = self
            .tokens
            .peek()
            .ok_or_else(|| Error::expected_token(&self.checked_tokens))?;
        Op::try_from(token.kind)
            .map(|op| (op, token.line))
            .map_err(|_| Error::unexpected_token(token, &self.checked_tokens))
    }

    fn consume_semicolon_or_eof(&mut self) -> Result<(), Error> {
        if self.peek().is_some() {
            self.consume_if(&[TT::SEMICOLON])?;
        }
        Ok(())
    }

    fn consume_token_value(&mut self) -> Result<(TV<'a>, usize), Error> {
        let token = self.consume_if(ValueTokenTypes::VARIANTS)?;
        let line = token.line;
        let value = token
            .token_value
            .expect("All value token types to have an associated value");
        Ok((value, line))
    }

    fn unexpected(&self, token: &Token<'a>) -> Error {
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
    tokens: TokenCursor<'a>,
    loop_depth: usize,
}

pub fn parse(tokens: Vec<Token>) -> Result<Vec<Statement>, StageError> {
    Ok(Parser::new(tokens).parse()?)
}

impl<'a> Parser<'a> {
    pub fn new(tokens: Vec<Token<'a>>) -> Self {
        Parser {
            tokens: TokenCursor::new(TokenIter(tokens.into_iter())),
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
        if self.tokens.consume_if(&[TT::LEFT_BRACE]).is_ok() {
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
            && next_token.kind != TT::RIGHT_BRACE
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

        if let Err(e) = self.tokens.consume_if(&[TT::RIGHT_BRACE]) {
            errors.push(e);
        }

        if !errors.is_empty() {
            return Err(Error::block_error(errors));
        }
        Ok(Statement::Group(members))
    }

    fn function_declaration(
        &mut self,
        line: usize,
    ) -> Result<Statement<'a>, Error> {
        debug!("Function");
        let is_named =
            matches!(self.tokens.peek(), Some(t) if t.kind == TT::IDENTIFIER);
        if is_named {
            let name = self.tokens.consume_name()?;
            Ok(Statement::FunctionDeclaration(self.function(name, line)?))
        } else {
            Ok(Statement::Expression(Expr::lambda(
                self.function("lambda", line)?,
                line,
            )))
        }
    }

    fn function(
        &mut self,
        name: &'a str,
        line: usize,
    ) -> Result<Function<'a>, Error> {
        self.tokens.consume_if(&[TT::LEFT_PAREN])?;

        let mut params = vec![];
        while let Ok(param) = self.tokens.consume_name() {
            params.push(param);
            if params.len() >= 255 {
                return Err(Error::too_many_arguments(line, true));
            }
            if self.tokens.consume_if(&[TT::COMMA]).is_err() {
                break;
            }
        }

        self.tokens.consume_if(&[TT::RIGHT_PAREN])?;
        self.tokens.consume_if(&[TT::LEFT_BRACE])?;
        let body = Box::new(self.block()?);
        Ok(Function {
            name,
            params,
            body: FunctionKind::Lox(body),
        })
    }

    fn keyword(&mut self) -> Result<Statement<'a>, Error> {
        debug!("Keyword");
        let Ok((keyword, line)) = self.tokens.peek_token_subset() else {
            // no keyword token, treat as an expression statement
            let statement = self.expression(0).map(Statement::Expression)?;
            self.tokens.consume_semicolon_or_eof()?;
            return Ok(statement);
        };
        let token = self.tokens.next().expect("Unwrapping a peeked value");

        let statement = match keyword {
            Keyword::VAR => {
                let declaration = self.declaration()?;
                self.tokens.consume_semicolon_or_eof()?;
                declaration
            }
            Keyword::FUN => self.function_declaration(line)?,
            Keyword::PRINT => {
                debug!("Print");
                let statement = self.expression(0).map(Statement::Print)?;
                self.tokens.consume_semicolon_or_eof()?;
                statement
            }
            Keyword::IF => self.if_statement()?,
            Keyword::WHILE => self.while_statement()?,
            Keyword::FOR => self.for_statement()?,
            Keyword::BREAK => {
                if self.loop_depth != 0 {
                    return Err(self.tokens.unexpected(&token));
                }
                self.tokens.consume_semicolon_or_eof()?;
                Statement::Break
            }
            Keyword::RETURN => self.return_statement(line)?,
        };

        Ok(statement)
    }

    fn declaration(&mut self) -> Result<Statement<'a>, Error> {
        debug!("Declaration");
        let name = self.tokens.consume_name()?;
        let expression = if self.tokens.consume_if(&[TT::EQUAL]).is_ok() {
            Some(self.expression(0)?)
        } else {
            None
        };
        Ok(Statement::Declaration { name, expression })
    }

    fn if_statement(&mut self) -> Result<Statement<'a>, Error> {
        debug!("If");
        self.tokens.consume_if(&[TT::LEFT_PAREN])?;
        let condition = self.expression(0)?;
        self.tokens.consume_if(&[TT::RIGHT_PAREN])?;

        let true_branch = Box::new(self.statement()?);
        let false_branch = match self.tokens.consume_if(&[TT::ELSE]) {
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
        self.tokens.consume_if(&[TT::LEFT_PAREN])?;
        let condition = self.expression(0)?;
        self.tokens.consume_if(&[TT::RIGHT_PAREN])?;

        self.loop_depth += 1;
        let body = Box::new(self.statement()?);
        self.loop_depth -= 1;

        Ok(Statement::While { condition, body })
    }

    fn for_statement(&mut self) -> Result<Statement<'a>, Error> {
        debug!("For");
        self.tokens.consume_if(&[TT::LEFT_PAREN])?;

        let mut next_token = self
            .tokens
            .peek()
            .ok_or(Error::unexpected_eof("For statement"))?;
        let line = next_token.line;

        let initialiser = if let Some(token) = self.tokens.peek()
            && token.kind == TT::SEMICOLON
        {
            None
        } else if self.tokens.consume_if(&[TT::VAR]).is_ok() {
            Some(self.declaration()?)
        } else {
            Some(self.expression(0).map(Statement::Expression)?)
        };

        self.tokens.consume_if(&[TT::SEMICOLON])?;
        debug!("For condition");

        next_token = self
            .tokens
            .peek()
            .ok_or(Error::unexpected_eof("For statement"))?;
        let condition = match next_token.kind {
            TT::SEMICOLON => None,
            _ => Some(self.expression(0)?),
        };
        self.tokens.consume_if(&[TT::SEMICOLON])?;
        debug!("For increment");

        next_token = self
            .tokens
            .peek()
            .ok_or(Error::unexpected_eof("For statement"))?;
        let increment = match next_token.kind {
            TT::SEMICOLON => None,
            _ => Some(self.expression(0)?),
        };
        self.tokens.consume_if(&[TT::RIGHT_PAREN])?;
        debug!("For body");

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
    ) -> Result<Statement<'a>, Error> {
        let return_expr = match self
            .tokens
            .peek()
            .ok_or(Error::unexpected_eof("Return statement"))?
            .kind
        {
            TT::SEMICOLON => None,
            _ => Some(self.expression(0)?),
        };

        self.tokens.consume_if(&[TT::SEMICOLON])?;

        Ok(Statement::Return {
            line,
            value: return_expr,
        })
    }

    fn expression(
        &mut self,
        current_precedence: usize,
    ) -> Result<Expr<'a>, Error> {
        debug!("Expression - {current_precedence}");
        let lhs = self.build_logical(current_precedence)?;

        if let Ok(token) = self.tokens.consume_if(&[TT::EQUAL]) {
            debug!("Building an assignment from {token:?}");
            let rhs = self.expression(current_precedence)?;

            if let ExprKind::Identifier(name) = lhs.kind {
                return Ok(Expr::assignment(name, Box::new(rhs), token.line));
            }
            return Err(Error::invalid_assignment_target(token.line));
        }

        Ok(lhs)
    }

    pub fn build_logical(
        &mut self,
        current_precedence: usize,
    ) -> Result<Expr<'a>, Error> {
        debug!("Logical");
        let mut lhs = self.build_binary(current_precedence)?;

        while let Ok((infix, line)) = self.tokens.peek_token_subset() {
            let precedence = logical_precedence(infix);
            if precedence < current_precedence {
                break;
            }
            self.tokens.next();

            let rhs = self.build_logical(precedence)?;
            lhs = Expr::logical(Box::new(lhs), infix, Box::new(rhs), line);
        }
        Ok(lhs)
    }

    pub fn build_binary(
        &mut self,
        current_precedence: usize,
    ) -> Result<Expr<'a>, Error> {
        debug!("Binary");
        let mut lhs = self.parse_prefix()?;

        while let Ok((infix, line)) = self.tokens.peek_token_subset() {
            let (l_precedence, r_precedence) = infix_precedence(infix);
            if l_precedence < current_precedence {
                break;
            }
            self.tokens.next();

            let rhs = self.build_binary(r_precedence)?;
            lhs = Expr::binary(Box::new(lhs), infix, Box::new(rhs), line);
        }
        Ok(lhs)
    }

    pub fn parse_prefix(&mut self) -> Result<Expr<'a>, Error> {
        debug!("Prefix");

        if let Ok((unary_op, line)) = self.tokens.peek_token_subset() {
            self.tokens.next();
            let expr = Box::new(self.parse_prefix()?);
            return Ok(Expr::unary(unary_op, expr, line));
        }

        let mut expr =
            if let Ok((value, line)) = self.tokens.consume_token_value() {
                build_value(value, line)
            } else if self.tokens.consume_if(&[TT::LEFT_PAREN]).is_ok() {
                let inner = self.expression(0)?;
                let token = self.tokens.consume_if(&[TT::RIGHT_PAREN])?;
                Expr::grouping(Box::new(inner), token.line)
            } else if let Ok(token) = self.tokens.consume_if(&[TT::FUN]) {
                Expr::lambda(self.function("lambda", token.line)?, token.line)
            } else {
                return Err(self.tokens.failed_to_match());
            };

        while self.tokens.consume_if(&[TT::LEFT_PAREN]).is_ok() {
            expr = self.build_call(expr)?;
        }
        Ok(expr)
    }

    fn build_call(&mut self, callee: Expr<'a>) -> Result<Expr<'a>, Error> {
        debug!("Call");
        let mut arguments = Vec::new();

        while let Some(next) = self.tokens.peek()
            && next.kind != TT::RIGHT_PAREN
        {
            debug!("First call arg = {next:?}");
            arguments.push(self.expression(5)?);
            if arguments.len() >= 255 {
                return Err(Error::too_many_arguments(
                    arguments[arguments.len() - 1].line,
                    false,
                ));
            }
            if self.tokens.consume_if(&[TT::COMMA]).is_err() {
                break;
            }
        }

        self.tokens.consume_if(&[TT::RIGHT_PAREN])?;

        Ok(Expr::call(Box::new(callee), arguments))
    }

    fn synchronise(&mut self) {
        debug!("Synchronising {:?}", self.tokens.peek());
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
    debug!("Building value from: {value:?}");
    match value {
        TV::String(text) => Expr::literal(Value::String(text.to_owned()), line),
        TV::Number(number) => Expr::literal(Value::Number(number), line),
        TV::Bool(val) => Expr::literal(Value::Boolean(val), line),
        TV::Nil => Expr::literal(Value::Nil, line),
        TV::Identifier(name) => Expr::identifier(name, line),
        TV::Comment(..) => {
            unreachable!("Shouldn't be emitted")
        }
    }
}
