pub mod environment;
use log::trace;
use std::cell::RefCell;
use std::fmt::Debug;
pub mod evaluation_error;
mod globals;
use std::fmt::Write;
use std::rc::Rc;

use crate::evaluator::environment::Environment;
use crate::evaluator::environment::VariableBinding;
use crate::evaluator::evaluation_error::EvaluationError as Error;
use crate::evaluator::evaluation_error::EvaluationError::{
    Break, IncorrectArgumentCount, NonFunctionCalled,
    NoninstancePropertyAccess, Return, SuperClassMustBeClass,
    UndefinedClassProperty, UndefinedInstanceProperty, UndefinedVariable,
    UnsupportedBinaryOperand, UnsupportedUnaryOperand,
};

use crate::evaluator::environment::LookupMap;
use crate::expressions::FunctionValue;
use crate::expressions::Get;
use crate::expressions::Instance;
use crate::expressions::Set;
use crate::expressions::{
    Binary, BinaryOperator, Call, Expr, ExprKind, FunctionKind, FunctionType,
    Logical, LogicalOperator, Statement, Unary, UnaryOperator, Value,
};
use crate::token::Span;

pub fn evaluate<'a, W: Write + Debug>(
    statements: &Vec<Statement<'a>>,
    locals: LookupMap<'a>,
    writer: &mut W,
) -> Result<Option<Value<'a>>, Vec<Error<'a>>> {
    trace!("Begining eval: {statements:?}\nlocals: {locals:?}");
    let mut eval = Evaluator::new(locals, writer);
    eval.evaluate_statements(statements)
}

struct Evaluator<'a, 'b, W: Write + Debug> {
    env: Environment<'a>,
    output: &'b mut W,
}
impl<'a, 'b, W: Write + Debug> Evaluator<'a, 'b, W> {
    fn new(locals: LookupMap<'a>, output: &'b mut W) -> Self {
        Self {
            env: Environment::new(locals),
            output,
        }
    }

    fn evaluate_statements(
        &mut self,
        statements: &Vec<Statement<'a>>,
    ) -> Result<Option<Value<'a>>, Vec<Error<'a>>> {
        let mut errors = Vec::new();
        let mut result: Option<Value<'a>> = None;
        for statement in statements {
            match self.eval(statement) {
                Ok(value) => result = value,
                Err(e)
                    if errors.is_empty()
                        && matches!(e, Break | Return { .. }) =>
                {
                    return Err(vec![e]);
                }
                Err(e) => errors.push(e),
            }
        }
        if !errors.is_empty() {
            return Err(errors);
        }
        Ok(result)
    }

    fn eval(
        &mut self,
        statement: &Statement<'a>,
    ) -> Result<Option<Value<'a>>, Error<'a>> {
        trace!("Eval");
        match statement {
            Statement::Expression(expr) => {
                trace!("Expression");
                return self.visit(expr).map(Some);
            }
            Statement::Print(expr) => {
                trace!("Print");
                let output = self.visit(expr)?;
                writeln!(self.output, "{output}")
                    .expect("Write to sink failed");
            }
            Statement::Declaration {
                binding,
                expression,
            } => {
                trace!("Declaration");
                if let Some(expr) = expression {
                    let value = Some(self.visit(expr)?);
                    self.env.define(binding, value);
                } else {
                    self.env.define(binding, None);
                }
            }
            Statement::FunctionDeclaration(declaration) => {
                trace!("Function Declaration");
                self.env.define(
                    &declaration.binding,
                    Some(self.define_function(declaration)),
                );
            }
            Statement::Group(s) => {
                trace!("Group");
                self.env.narrow();
                let result = self.evaluate_statements(s);
                self.env.pop();
                result?;
            }
            Statement::If {
                condition,
                true_branch,
                false_branch,
            } => {
                trace!("If");
                if as_bool(&self.visit(condition)?) {
                    trace!("If success");
                    self.eval(true_branch)?;
                } else if let Some(branch) = false_branch {
                    self.eval(branch)?;
                }
            }
            Statement::While { condition, body } => {
                trace!("While");
                while as_bool(&self.visit(condition)?) {
                    trace!("Looping!");
                    let result = self.eval(body);
                    match result {
                        Err(Break) => return Ok(None),
                        Err(e) => return Err(e),
                        _ => {}
                    }
                }
            }
            Statement::Break => {
                trace!("Break");
                return Err(Break);
            }
            Statement::Return {
                span,
                value: value_expr,
            } => {
                trace!("Return");
                let return_value = match value_expr {
                    Some(expr) => self.visit(expr)?,
                    None => Value::Nil,
                };

                return Err(Return {
                    span: *span,
                    value: return_value,
                });
            }
            Statement::ClassDeclaration(class) => {
                trace!("Class Declaration");
                self.env
                    .define(&class.binding, Some(Value::Class(class.clone())));
            }
        }
        Ok(None)
    }

    fn define_function(&self, function: &FunctionType<'a>) -> Value<'a> {
        Value::Function(FunctionValue {
            function: function.clone(),
            closure: self.env.top().clone(),
        })
    }

    fn visit(&mut self, expr: &Expr<'a>) -> Result<Value<'a>, Error<'a>> {
        match &expr.kind {
            ExprKind::Literal(value) => Ok(value.clone()),
            ExprKind::Unary(unary) => self.visit_unary(unary, expr.span),
            ExprKind::Binary(binary) => self.visit_binary(binary, expr.span),
            ExprKind::Grouping(expr) => self.visit(expr),
            ExprKind::Identifier(name) => {
                let binding = VariableBinding {
                    name,
                    span: expr.span,
                };

                self.env.get(&binding)
            }
            ExprKind::Assignment(assignment) => {
                trace!("Assignment");
                let binding = VariableBinding {
                    name: assignment.name,
                    span: assignment.expr.span,
                };
                let value = self.visit(&assignment.expr)?;

                self.env
                    .update(&binding, value.clone())
                    .map_err(|()| UndefinedVariable(binding))?;
                Ok(value)
            }
            ExprKind::Logical(logical) => self.visit_logical(logical),
            ExprKind::Call(call) => self.visit_call(call, expr.span),
            ExprKind::Lambda(function) => Ok(self.define_function(function)),
            ExprKind::Get(get) => self.visit_get(get),
            ExprKind::Set(set) => self.visit_set(set),
            ExprKind::This(binding) => self.env.get(binding),
            ExprKind::Super { binding, target } => {
                let Value::Class(superclass) = self.env.get(binding)? else {
                    return Err(SuperClassMustBeClass(binding.span));
                };
                let Value::Instance(instance) =
                    self.env.get(&VariableBinding {
                        name: "this",
                        span: binding.span,
                    })?
                else {
                    unreachable!("This to be bound by the resolver");
                };

                let method = superclass.find_method(target.name, &self.env)?;

                method.map_or_else(
                    || {
                        Err(UndefinedClassProperty {
                            class: superclass,
                            target: target.clone(),
                        })
                    },
                    |method| Ok(Value::Function(method.bound(instance))),
                )
            }
        }
    }

    fn visit_unary(
        &mut self,
        unary: &Unary<'a>,
        span: Span,
    ) -> Result<Value<'a>, Error<'a>> {
        let value = self.visit(&unary.expr)?;

        match unary.operator {
            UnaryOperator::Minus => match value {
                Value::Number(num) => Ok(Value::Number(-num)),
                _ => Err(UnsupportedUnaryOperand {
                    expr_type: value.type_name(),
                    operator: unary.operator,
                    span,
                }),
            },
            UnaryOperator::Bang => Ok(Value::Boolean(!as_bool(&value))),
        }
    }

    #[allow(
        clippy::string_add,
        reason = "Do not want to modify the original string inplace"
    )]
    fn visit_binary(
        &mut self,
        binary: &Binary<'a>,
        span: Span,
    ) -> Result<Value<'a>, Error<'a>> {
        trace!("Binary");
        let left_value = self.visit(&binary.left)?;
        let right_value = self.visit(&binary.right)?;

        match binary.operator {
            BinaryOperator::EqualEqual => {
                return Ok(Value::Boolean(is_equal(&left_value, &right_value)));
            }
            BinaryOperator::BangEqual => {
                return Ok(Value::Boolean(!is_equal(
                    &left_value,
                    &right_value,
                )));
            }
            BinaryOperator::Plus => {
                if let Value::String(lhs_string) = left_value {
                    return Ok(Value::String(
                        lhs_string + &right_value.cast_to_string(),
                    ));
                } else if let Value::String(rhs_string) = right_value {
                    return Ok(Value::String(
                        left_value.cast_to_string() + &rhs_string,
                    ));
                }
            }
            _ => {}
        }

        let left_type = left_value.type_name();

        if let Value::Number(lhs) = left_value
            && let Value::Number(rhs) = right_value
            && let Some(value) =
                numeric_binary_operations(lhs, binary.operator, rhs)
        {
            return Ok(value);
        }
        Err(UnsupportedBinaryOperand {
            lhs_type: left_type,
            operator: binary.operator,
            rhs_type: right_value.type_name(),
            span,
        })
    }

    fn visit_logical(
        &mut self,
        logical: &Logical<'a>,
    ) -> Result<Value<'a>, Error<'a>> {
        let lhs_value = self.visit(&logical.left)?;
        let lhs_truthy = as_bool(&lhs_value);

        match logical.operator {
            LogicalOperator::Or if !lhs_truthy => self.visit(&logical.right),
            LogicalOperator::And if lhs_truthy => self.visit(&logical.right),
            _ => Ok(lhs_value),
        }
    }

    fn visit_call(
        &mut self,
        call: &Call<'a>,
        span: Span,
    ) -> Result<Value<'a>, Error<'a>> {
        trace!("Call");
        let callee = self.visit(&call.callee)?;

        let function_value = match callee {
            Value::Function(func) => func,
            Value::Class(class) => class.constructor(),
            _ => {
                return Err(NonFunctionCalled { span });
            }
        };

        let arguments: Vec<Value<'a>> = call
            .arguments
            .iter()
            .map(|arg| self.visit(arg))
            .collect::<Result<_, _>>()?;

        self.call_function(function_value, arguments, span)
    }

    fn call_function(
        &mut self,
        function_value: FunctionValue<'a>,
        arguments: Vec<Value<'a>>,
        span: Span,
    ) -> Result<Value<'a>, Error<'a>> {
        let func = function_value.function;
        let closure = &function_value.closure;

        if arguments.len() != func.params.len() {
            return Err(IncorrectArgumentCount {
                span,
                expected_arguments: func.params.len(),
                received_arguments_count: arguments.len(),
            });
        }

        match func.body {
            FunctionKind::Rust(function) => Ok(function(arguments)),
            FunctionKind::Constructor(class) => {
                let instance = Rc::new(RefCell::new(Instance::new(class)));
                let Some(Value::Function(init)) =
                    instance.borrow_mut().get("init", &self.env)?
                else {
                    return Ok(Value::Instance(instance));
                };

                self.call_function(
                    init.bind(Rc::clone(&instance)),
                    arguments,
                    span,
                )?;
                Ok(Value::Instance(instance))
            }
            FunctionKind::Lox(statement) => {
                self.env.push(closure.clone());
                self.env.narrow();

                for (name, value) in func.params.iter().zip(arguments) {
                    self.env.define(name, Some(value));
                }
                let result = self.eval(&statement);
                self.env.pop();
                self.env.pop();

                match result {
                    Ok(_) => Ok(Value::Nil),
                    Err(Return { value, .. }) => Ok(value),
                    Err(e) => Err(e),
                }
            }
            FunctionKind::CustomConstructor(statement) => {
                self.env.push(closure.clone());
                self.env.narrow();

                for (name, value) in func.params.iter().zip(arguments) {
                    self.env.define(name, Some(value));
                }
                let eval_result = self.eval(&statement);

                self.env.pop();
                let instance = self.env.get_local("this");
                self.env.pop();

                match eval_result {
                    Ok(_) | Err(Return { .. }) => {
                        Ok(instance.expect("Constructor guarantees instance"))
                    }
                    Err(e) => Err(e),
                }
            }
        }
    }

    fn visit_get(&mut self, get: &Get<'a>) -> Result<Value<'a>, Error<'a>> {
        trace!("Get");
        let target = get.target.clone();

        let Value::Instance(instance) = self.visit(&get.object)? else {
            return Err(NoninstancePropertyAccess {
                object: get.object.span,
                attribute: get.target.span,
            });
        };

        let attribute = instance.borrow().get(target.name, &self.env)?.clone();

        match attribute {
            Some(Value::Function(method)) => {
                Ok(Value::Function(method.bind(instance)))
            }
            Some(val) => Ok(val.clone()),
            None => Err(UndefinedInstanceProperty { instance, target }),
        }
    }

    fn visit_set(&mut self, set: &Set<'a>) -> Result<Value<'a>, Error<'a>> {
        trace!("Set");
        let target = set.target.clone();

        let Value::Instance(instance) = self.visit(&set.object)? else {
            return Err(NoninstancePropertyAccess {
                object: set.object.span,
                attribute: set.target.span,
            });
        };

        let value = self.visit(&set.value)?;

        instance.borrow_mut().set(target.name, value.clone());

        Ok(value)
    }
}

#[allow(clippy::float_cmp, reason = "User is trying to float cmp")]
fn numeric_binary_operations<'a>(
    lhs: f64,
    operator: BinaryOperator,
    rhs: f64,
) -> Option<Value<'a>> {
    let result = match operator {
        BinaryOperator::Minus => Value::Number(lhs - rhs),
        BinaryOperator::Slash => Value::Number(lhs / rhs),
        BinaryOperator::Star => Value::Number(lhs * rhs),
        BinaryOperator::Plus => {
            trace!("Summation of {lhs} and {rhs}");
            Value::Number(lhs + rhs)
        }
        BinaryOperator::Greater => Value::Boolean(lhs > rhs),
        BinaryOperator::GreaterEqual => Value::Boolean(lhs >= rhs),
        BinaryOperator::Less => Value::Boolean(lhs < rhs),
        BinaryOperator::LessEqual => Value::Boolean(lhs <= rhs),
        _ => {
            return None;
        }
    };
    Some(result)
}

pub const fn as_bool(value: &Value) -> bool {
    match *value {
        Value::Nil => false,
        Value::Boolean(bool_value) => bool_value,
        _ => true,
    }
}

#[allow(clippy::float_cmp, reason = "User is trying to float cmp")]
fn is_equal(left_value: &Value, right_value: &Value) -> bool {
    match left_value {
        Value::String(lhs) => {
            if let Value::String(rhs) = right_value {
                return lhs == rhs;
            }
        }
        Value::Number(lhs) => {
            if let Value::Number(rhs) = *right_value {
                return *lhs == rhs;
            }
        }
        Value::Boolean(lhs) => {
            if let Value::Boolean(rhs) = *right_value {
                return *lhs == rhs;
            }
        }
        Value::Nil => {
            return matches!(*right_value, Value::Nil);
        }
        Value::Function { .. } | Value::Class(..) | Value::Instance(..) => {
            return false;
        }
    }
    false
}
