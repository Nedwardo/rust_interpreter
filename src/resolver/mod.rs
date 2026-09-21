mod resolver_error;
use crate::error_utils::StageError;
use crate::evaluator::environment::{LookupMap, VariableBinding};
use crate::resolver::resolver_error::ResolverError;
use std::collections::HashMap;

use crate::expressions::Statement::{
    Break, Declaration, Expression, FunctionDeclaration, Group, If, Print,
    Return, While,
};
use crate::expressions::{
    Assignment, Binary, Call, Expr, ExprKind, Function, FunctionKind, Logical,
    Statement, Unary,
};

type Scope<'a> = HashMap<&'a str, bool>;
pub struct Resolver<'a> {
    symbol_table: Vec<Scope<'a>>,
}

pub fn resolve<'a>(
    statements: &Vec<Statement<'a>>,
) -> Result<LookupMap<'a>, StageError> {
    let mut resolver = Resolver::new();
    Ok(resolver.resolve_statements(statements)?)
}

impl<'a> Resolver<'a> {
    pub const fn new() -> Self {
        Self {
            symbol_table: Vec::new(),
        }
    }

    fn narrow(&mut self) {
        self.symbol_table.push(Scope::new());
    }

    fn widen(&mut self) {
        self.symbol_table.pop();
    }

    fn declare(&mut self, name: &'a str) {
        if let Some(scope) = self.symbol_table.last_mut() {
            let _ = scope.insert(name, false);
        }
    }

    fn define(&mut self, name: &'a str) {
        if let Some(scope) = self.symbol_table.last_mut() {
            let _ = scope.insert(name, true);
        }
    }

    fn resolve_statements(
        &mut self,
        statements: &Vec<Statement<'a>>,
    ) -> Result<LookupMap<'a>, ResolverError> {
        let mut locals: LookupMap<'a> = HashMap::new();
        for statement in statements {
            locals.extend(self.resolve_statement(statement)?);
        }
        Ok(locals)
    }

    fn resolve_statement(
        &mut self,
        statement: &Statement<'a>,
    ) -> Result<LookupMap<'a>, ResolverError> {
        match statement {
            Break => Ok(HashMap::new()),
            Declaration { name, expression } => {
                self.declare(name);
                let binding = expression.as_ref().map_or_else(
                    || Ok(HashMap::new()),
                    |expr| self.resolve_expression(expr),
                );
                self.define(name);
                binding
            }
            Expression(expr) | Print(expr) => self.resolve_expression(expr),
            FunctionDeclaration(Function {
                name: _,
                body,
                params,
            }) => {
                self.narrow();
                for token in params {
                    self.declare(token);
                    self.define(token);
                }
                let binding = if let FunctionKind::Lox(lox_body) = body {
                    self.resolve_statement(lox_body)
                } else {
                    Ok(HashMap::new())
                };
                self.widen();
                binding
            }
            Group(statements) => {
                self.narrow();
                let result = self.resolve_statements(statements);
                self.widen();
                result
            }
            If {
                condition,
                true_branch,
                false_branch,
            } => {
                let mut locals = self.resolve_expression(condition)?;
                locals.extend(self.resolve_statement(true_branch)?);
                if let Some(other_branch) = false_branch {
                    locals.extend(self.resolve_statement(other_branch)?);
                }
                Ok(locals)
            }

            Return { line: _, value } => value.as_ref().map_or_else(
                || Ok(HashMap::new()),
                |expr| self.resolve_expression(expr),
            ),
            While { condition, body } => {
                let mut locals = self.resolve_expression(condition)?;
                locals.extend(self.resolve_statement(body)?);
                Ok(locals)
            }
        }
    }

    fn resolve_expression(
        &mut self,
        expr: &Expr<'a>,
    ) -> Result<LookupMap<'a>, ResolverError> {
        match &expr.kind {
            ExprKind::Assignment(Assignment { name, expr }) => {
                let binding = VariableBinding {
                    line: expr.line,
                    name,
                };
                let mut locals = self.resolve_expression(expr)?;
                let depth = self.resolve_local(name);
                locals.insert(binding, depth);
                Ok(locals)
            }
            ExprKind::Binary(Binary {
                left,
                operator: _,
                right,
            })
            | ExprKind::Logical(Logical {
                left,
                operator: _,
                right,
            }) => {
                let mut locals = self.resolve_expression(left)?;
                locals.extend(self.resolve_expression(right)?);
                Ok(locals)
            }
            ExprKind::Call(Call { callee, arguments }) => {
                let mut locals = self.resolve_expression(callee)?;
                for arg in arguments {
                    locals.extend(self.resolve_expression(arg)?);
                }
                Ok(locals)
            }
            ExprKind::Grouping(child) => self.resolve_expression(child),
            ExprKind::Identifier(name) => {
                if self.symbol_table.last().map(|scope| scope.get(name))
                    == Some(Some(&false))
                {
                    return Err(
                        ResolverError::VariableReferencedInInitalisation {
                            name: name.to_string(),
                            line: expr.line,
                        },
                    );
                }
                let binding = VariableBinding {
                    line: expr.line,
                    name,
                };
                Ok(HashMap::from([(binding, self.resolve_local(name))]))
            }
            ExprKind::Lambda(Function {
                name: _,
                body,
                params,
            }) => {
                self.narrow();
                for token in params {
                    self.declare(token);
                    self.define(token);
                }

                let mut locals = HashMap::new();
                if let FunctionKind::Lox(lox_body) = body {
                    locals.extend(self.resolve_statement(lox_body)?);
                }
                self.widen();
                Ok(locals)
            }
            ExprKind::Unary(Unary { operator: _, expr }) => {
                self.resolve_expression(expr)
            }
            ExprKind::Literal(_) => Ok(HashMap::new()),
        }
    }

    fn resolve_local(&self, name: &'a str) -> usize {
        for (index, scope) in self.symbol_table.iter().enumerate().rev() {
            if scope.get(name).is_some() {
                return index;
            }
        }
        panic!("Should not be reachable, will return to this");
    }
}
