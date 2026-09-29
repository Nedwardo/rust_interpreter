mod resolver_error;
use log::trace;

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

#[derive(Debug, PartialEq, Clone)]
enum FunctionType {
    Function,
}

pub struct Resolver<'a> {
    symbol_table: Vec<Scope<'a>>,
    current_function_state: Option<FunctionType>,
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
            current_function_state: None,
        }
    }

    fn narrow(&mut self) {
        trace!("Narrowing");
        self.symbol_table.push(Scope::new());
    }

    fn widen(&mut self) {
        self.symbol_table.pop();
    }

    fn declare(
        &mut self,
        binding: &VariableBinding<'a>,
    ) -> Result<(), ResolverError> {
        trace!("Declaring {:#?}", binding.name);
        if let Some(scope) = self.symbol_table.last_mut() {
            if scope.contains_key(binding.name) {
                return Err(ResolverError::VariableAlreadyExists {
                    name: binding.name.to_owned(),
                    span: binding.span,
                });
            }
            let _ = scope.insert(binding.name, false);
            trace!("Success");
        }
        Ok(())
    }

    fn define(&mut self, name: &'a str) {
        trace!("Defining {name}");
        if let Some(scope) = self.symbol_table.last_mut() {
            let _ = scope.insert(name, true);
            trace!("Success");
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
            Declaration {
                binding,
                expression,
            } => {
                self.declare(binding)?;
                let value = expression.as_ref().map_or_else(
                    || Ok(HashMap::new()),
                    |expr| self.resolve_expression(expr),
                );
                self.define(binding.name);
                value
            }
            Expression(expr) | Print(expr) => self.resolve_expression(expr),
            FunctionDeclaration(function) => self.resolve_function(function),
            Group(statements) => {
                trace!("Group: {:?}", self.symbol_table);
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

            Return { span, value } => {
                if self.current_function_state == Some(FunctionType::Function) {
                    value.as_ref().map_or_else(
                        || Ok(HashMap::new()),
                        |expr| self.resolve_expression(expr),
                    )
                } else {
                    Err(ResolverError::ReturnFromTopLevel(*span))
                }
            }
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
                    span: expr.span,
                    name,
                };
                let mut locals = self.resolve_expression(expr)?;
                trace!("Creating assignment for {binding:?}");
                if let Some(depth) = self.resolve_local(name) {
                    locals.insert(binding, depth);
                }
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
                            span: expr.span,
                        },
                    );
                }
                let binding = VariableBinding {
                    span: expr.span,
                    name,
                };
                trace!("Resolving local: {binding:?}");
                Ok(self
                    .resolve_local(name)
                    .map_or_else(HashMap::new, |depth| {
                        HashMap::from([(binding, depth)])
                    }))
            }
            ExprKind::Lambda(function) => self.resolve_function(function),
            ExprKind::Unary(Unary { operator: _, expr }) => {
                self.resolve_expression(expr)
            }
            ExprKind::Literal(_) => Ok(HashMap::new()),
        }
    }

    fn resolve_function(
        &mut self,
        function: &Function<'a>,
    ) -> Result<LookupMap<'a>, ResolverError> {
        self.declare(&function.binding)?;
        self.define(function.binding.name);

        trace!("res func");
        self.narrow();
        for token in &function.params {
            self.declare(token)?;
            self.define(token.name);
        }
        let binding = if let FunctionKind::Lox(lox_body) = &function.body {
            let prior_function_state = self.current_function_state.clone();
            self.current_function_state = Some(FunctionType::Function);

            let lookup_map = self.resolve_statement(lox_body);
            self.current_function_state = prior_function_state;
            lookup_map
        } else {
            Ok(HashMap::new())
        };
        self.widen();
        binding
    }

    fn resolve_local(&self, name: &'a str) -> Option<usize> {
        let symbol_table_len = self.symbol_table.len();
        for (index, scope) in self.symbol_table.iter().enumerate().rev() {
            if scope.get(name).is_some() {
                trace!(
                    "Resolved {name}, with {:?}, giving dist of {:?}",
                    self.symbol_table,
                    symbol_table_len - index - 1
                );
                return Some(symbol_table_len - index - 1);
            }
        }
        None
    }
}
