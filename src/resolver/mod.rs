mod resolver_error;
use log::trace;

use crate::error_utils::StageError;
use crate::evaluator::environment::{LookupMap, VariableBinding};
use crate::resolver::resolver_error::ResolverError as Error;
use std::collections::HashMap;

use crate::expressions::Statement::{
    Break, ClassDeclaration, Declaration, Expression, FunctionDeclaration,
    Group, If, Print, Return, While,
};
use crate::expressions::{
    Assignment, Binary, Call, Class, Expr, ExprKind, FunctionKind,
    FunctionType, Get, Logical, Set, Statement, Unary,
};

#[derive(Debug)]
struct Scope<'a>(HashMap<&'a str, (bool, VariableBinding<'a>)>);

impl<'a> Scope<'a> {
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    fn get(&self, name: &'a str) -> Option<&bool> {
        self.0.get(name).map(|(v, ..)| v)
    }

    fn undefined_values(&self) -> Vec<VariableBinding<'a>> {
        self.0
            .values()
            .filter_map(|(v, binding)| (!v).then_some(binding.clone()))
            .collect()
    }

    fn contains(&self, key: &'a str) -> bool {
        self.0.contains_key(key)
    }

    fn insert(
        &mut self,
        binding: VariableBinding<'a>,
        value: bool,
    ) -> Option<bool> {
        self.0
            .insert(binding.name, (value, binding))
            .map(|(v, ..)| v)
    }
}

#[derive(Debug, PartialEq, Clone)]
enum FunctionState {
    None,
    Function,
    Method,
    Initializer,
}

#[derive(Debug, PartialEq, Clone, Copy)]
enum ClassState {
    None,
    Class,
}

pub struct Resolver<'a> {
    symbol_table: Vec<Scope<'a>>,
    current_function_state: FunctionState,
    current_class_state: ClassState,
}

pub fn resolve<'a>(
    statements: &Vec<Statement<'a>>,
) -> Result<LookupMap<'a>, StageError> {
    let mut resolver = Resolver::new();
    let lm = resolver.resolve_statements(statements)?;
    trace!("Ending resolve!!!\n");
    trace!("{lm:?}");
    Ok(lm)
}

impl<'a> Resolver<'a> {
    pub const fn new() -> Self {
        Self {
            symbol_table: Vec::new(),
            current_function_state: FunctionState::None,
            current_class_state: ClassState::None,
        }
    }

    fn narrow(&mut self) {
        trace!("Narrowing");
        self.symbol_table.push(Scope::new());
    }

    fn widen(&mut self) -> Result<(), Error<'a>> {
        let symbols = self
            .symbol_table
            .pop()
            .expect("Widen to only be used after a narrow");

        let undefined_values: Vec<_> = symbols.undefined_values();

        if undefined_values.is_empty() {
            Ok(())
        } else {
            Err(Error::unused_values(&undefined_values))
        }
    }

    fn declare(
        &mut self,
        binding: VariableBinding<'a>,
    ) -> Result<(), Error<'a>> {
        trace!("Declaring {binding:#?}");

        if let Some(scope) = self.symbol_table.last_mut() {
            if scope.contains(binding.name) {
                return Err(Error::VariableAlreadyExists {
                    name: binding.name.to_owned(),
                    span: binding.span,
                });
            }
            let _ = scope.insert(binding, false);
            trace!("Success");
        }
        Ok(())
    }

    fn define(&mut self, binding: VariableBinding<'a>) {
        trace!("Defining {binding:?}");
        if let Some(scope) = self.symbol_table.last_mut() {
            let _ = scope.insert(binding, true);
        }
    }

    fn resolve_statements(
        &mut self,
        statements: &Vec<Statement<'a>>,
    ) -> Result<LookupMap<'a>, Error<'a>> {
        let mut locals: LookupMap<'a> = HashMap::new();
        for statement in statements {
            locals.extend(self.resolve_statement(statement)?);
        }
        Ok(locals)
    }

    fn resolve_statement(
        &mut self,
        statement: &Statement<'a>,
    ) -> Result<LookupMap<'a>, Error<'a>> {
        match statement {
            Break => Ok(HashMap::new()),
            Declaration {
                binding,
                expression,
            } => {
                self.declare(binding.clone())?;
                let value = expression.as_ref().map_or_else(
                    || Ok(HashMap::new()),
                    |expr| self.resolve_expression(expr),
                );
                self.define(binding.clone());
                value
            }
            Expression(expr) | Print(expr) => self.resolve_expression(expr),
            FunctionDeclaration(function) => {
                self.resolve_function(function, FunctionState::Function)
            }
            Group(statements) => {
                trace!("Group: {:?}", self.symbol_table);
                self.narrow();
                let result = self.resolve_statements(statements);
                self.widen()?;
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

            Return { span, value } => match self.current_function_state {
                FunctionState::Function | FunctionState::Method => {
                    value.as_ref().map_or_else(
                        || Ok(HashMap::new()),
                        |expr| self.resolve_expression(expr),
                    )
                }
                FunctionState::Initializer if value.is_none() => {
                    Ok(HashMap::new())
                }
                FunctionState::Initializer => Err(Error::ReturnFromInit(*span)),
                FunctionState::None => Err(Error::ReturnFromTopLevel(*span)),
            },
            While { condition, body } => {
                let mut locals = self.resolve_expression(condition)?;
                locals.extend(self.resolve_statement(body)?);
                Ok(locals)
            }
            ClassDeclaration(class) => self.resolve_class(class),
        }
    }

    fn resolve_expression(
        &mut self,
        expr: &Expr<'a>,
    ) -> Result<LookupMap<'a>, Error<'a>> {
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
            ExprKind::Binary(Binary { left, right, .. })
            | ExprKind::Logical(Logical { left, right, .. }) => {
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
                    return Err(Error::VariableReferencedInInitalisation {
                        name: name.to_string(),
                        span: expr.span,
                    });
                }
                let binding = VariableBinding {
                    span: expr.span,
                    name,
                };
                Ok(self
                    .resolve_local(name)
                    .map_or_else(HashMap::new, |depth| {
                        HashMap::from([(binding, depth)])
                    }))
            }
            ExprKind::Lambda(function) => {
                self.resolve_function(function, FunctionState::Function)
            }
            ExprKind::Unary(Unary { expr, .. }) => {
                self.resolve_expression(expr)
            }
            ExprKind::Literal(_) => Ok(HashMap::new()),
            ExprKind::Get(Get { object, .. }) => {
                self.resolve_expression(object)
            }
            ExprKind::Set(Set { object, value, .. }) => {
                self.resolve_expression(value)?;
                self.resolve_expression(object)
            }
            ExprKind::This(binding) => {
                if matches!(self.current_class_state, ClassState::Class) {
                    Ok(self
                        .resolve_local(binding.name)
                        .map_or_else(HashMap::new, |depth| {
                            HashMap::from([(binding.clone(), depth)])
                        }))
                } else {
                    Err(Error::ThisOutsideOfClass(binding.span))
                }
            }
            ExprKind::Super { binding, .. } => {
                let mut lookup_map = self
                    .resolve_local(binding.name)
                    .map_or_else(HashMap::new, |depth| {
                        HashMap::from([(binding.clone(), depth)])
                    });

                let binding = VariableBinding {
                    span: binding.span,
                    name: "this",
                };
                let depth = self
                    .resolve_local("this")
                    .expect("If super is defined, this must be too");

                lookup_map.insert(binding, depth);

                Ok(lookup_map)
            }
        }
    }

    fn resolve_function(
        &mut self,
        function: &FunctionType<'a>,
        function_kind: FunctionState,
    ) -> Result<LookupMap<'a>, Error<'a>> {
        self.declare(function.binding.clone())?;
        self.define(function.binding.clone());

        self.narrow();
        for token in &function.params {
            self.declare(token.clone())?;
            self.define(token.clone());
        }
        let binding = match &function.body {
            FunctionKind::Lox(lox_body)
            | FunctionKind::CustomConstructor(lox_body) => {
                let prior_function_state = self.current_function_state.clone();
                self.current_function_state = function_kind;

                let lookup_map = self.resolve_statement(lox_body);
                self.current_function_state = prior_function_state;
                lookup_map
            }
            _ => Ok(HashMap::new()),
        };
        self.widen()?;
        binding
    }

    fn resolve_class(
        &mut self,
        class: &Class<'a>,
    ) -> Result<LookupMap<'a>, Error<'a>> {
        let enclosing_class_state = self.current_class_state;
        self.current_class_state = ClassState::Class;

        self.declare(class.binding.clone())?;
        self.define(class.binding.clone());

        let mut fields = HashMap::new();

        if let Some(superclass) = &class.superclass_binding {
            if superclass.name == class.binding.name {
                return Err(Error::ClassCantInheritFromItself(
                    class.binding.span,
                ));
            }
            fields.extend(self.resolve_expression(&Expr {
                span: superclass.span,
                kind: ExprKind::Identifier(superclass.name),
            })?);

            self.narrow();
            self.define(VariableBinding {
                name: "super",
                span: superclass.span,
            });
        }

        self.narrow();
        self.define(class.this_binding());

        for method in &class.methods {
            let function_state = if method.binding.name == "init" {
                FunctionState::Initializer
            } else {
                FunctionState::Method
            };
            fields.extend(self.resolve_function(method, function_state)?);
        }
        self.widen()?;
        if class.superclass_binding.is_some() {
            self.widen()?;
        }

        self.current_class_state = enclosing_class_state;

        trace!("{fields:?}");
        Ok(fields)
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
