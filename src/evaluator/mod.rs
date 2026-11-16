pub mod environment;
use log::trace;
use std::cell::RefCell;
use std::fmt::Debug;
pub mod evaluation_error;
mod globals;
use std::fmt::Write;
use std::rc::Rc;

use crate::evaluator::environment::VariableBinding;
use crate::evaluator::environment::{Environment, GetError};
use crate::evaluator::evaluation_error::EvaluationError as Error;
use crate::evaluator::evaluation_error::EvaluationError::{
    Break, Return, UndefinedVariable, UninitialisedVariable,
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
    evaluate_statements(statements, &mut Environment::new(locals), writer)
}

fn evaluate_statements<'a, W: Write + Debug>(
    statements: &Vec<Statement<'a>>,
    env: &mut Environment<'a>,
    writer: &mut W,
) -> Result<Option<Value<'a>>, Vec<Error<'a>>> {
    let mut errors = Vec::new();
    let mut result: Option<Value<'a>> = None;
    for statement in statements {
        match eval(statement, env, writer) {
            Ok(value) => result = value,
            Err(e)
                if errors.is_empty() && matches!(e, Break | Return { .. }) =>
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

#[allow(clippy::result_large_err, reason = "Return is handled by error path")]
fn eval<'a, W: Write + Debug>(
    statement: &Statement<'a>,
    env: &mut Environment<'a>,
    writer: &mut W,
) -> Result<Option<Value<'a>>, Error<'a>> {
    trace!("Eval");
    match statement {
        Statement::Expression(expr) => {
            trace!("Expression");
            return visit(expr, env, writer).map(Some);
        }
        Statement::Print(expr) => {
            trace!("Print");
            let output = visit(expr, env, writer)?;
            writeln!(writer, "{output}").expect("Write to sink failed");
        }
        Statement::Declaration {
            binding,
            expression,
        } => {
            trace!("Declaration");
            if let Some(expr) = expression {
                let value = Some(visit(expr, env, writer)?);
                env.define(binding, value);
            } else {
                env.define(binding, None);
            }
        }
        Statement::FunctionDeclaration(declaration) => {
            trace!("Function Declaration");
            env.define(
                &declaration.binding,
                Some(define_function(declaration, env)),
            );
        }
        Statement::Group(s) => {
            trace!("Group");
            env.narrow();
            let result = evaluate_statements(s, env, writer);
            env.pop();
            result?;
        }
        Statement::If {
            condition,
            true_branch,
            false_branch,
        } => {
            trace!("If");
            if as_bool(&visit(condition, env, writer)?) {
                trace!("If success");
                eval(true_branch, env, writer)?;
            } else if let Some(branch) = false_branch {
                eval(branch, env, writer)?;
            }
        }
        Statement::While { condition, body } => {
            trace!("While");
            while as_bool(&visit(condition, env, writer)?) {
                trace!("Looping!");
                let result = eval(body, env, writer);
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
                Some(expr) => visit(expr, env, writer)?,
                None => Value::Nil,
            };

            return Err(Return {
                span: *span,
                value: return_value,
            });
        }
        Statement::ClassDeclaration(class) => {
            trace!("Class Declaration");
            env.define(&class.binding, Some(Value::Class(class.clone())));
        }
    }
    Ok(None)
}

fn define_function<'a>(
    function: &FunctionType<'a>,
    env: &Environment<'a>,
) -> Value<'a> {
    Value::Function(FunctionValue {
        function: function.clone(),
        closure: env.top().clone(),
    })
}

#[allow(clippy::result_large_err, reason = "Return is handled by error path")]
fn visit<'a, W: Write + Debug>(
    expr: &Expr<'a>,
    env: &mut Environment<'a>,
    writer: &mut W,
) -> Result<Value<'a>, Error<'a>> {
    match &expr.kind {
        ExprKind::Literal(value) => Ok(value.clone()),
        ExprKind::Unary(unary) => visit_unary(unary, expr.span, env, writer),
        ExprKind::Binary(binary) => {
            visit_binary(binary, expr.span, env, writer)
        }
        ExprKind::Grouping(expr) => visit(expr, env, writer),
        ExprKind::Identifier(name) => {
            let binding = VariableBinding {
                name,
                span: expr.span,
            };

            env.get(&binding).map_err(|err| match err {
                GetError::Undefined => {
                    trace!("{env}");
                    UndefinedVariable(binding)
                }
                GetError::Uninitalised => UninitialisedVariable(binding),
            })
        }
        ExprKind::Assignment(assignment) => {
            trace!("Assignment");
            let binding = VariableBinding {
                name: assignment.name,
                span: assignment.expr.span,
            };
            let value = visit(&assignment.expr, env, writer)?;

            env.update(&binding, value.clone())
                .map_err(|()| UndefinedVariable(binding))?;
            Ok(value)
        }
        ExprKind::Logical(logical) => visit_logical(logical, env, writer),
        ExprKind::Call(call) => visit_call(call, expr.span, env, writer),
        ExprKind::Lambda(function) => Ok(define_function(function, env)),
        ExprKind::Get(get) => visit_get(get, env, writer),
        ExprKind::Set(set) => visit_set(set, env, writer),
        ExprKind::This(binding) => env.get(binding).map_err(|err| match err {
            GetError::Undefined => UndefinedVariable(binding.clone()),
            GetError::Uninitalised => UninitialisedVariable(binding.clone()),
        }),
    }
}

#[allow(clippy::result_large_err, reason = "Return is handled by error path")]
fn visit_unary<'a, W: Write + Debug>(
    unary: &Unary<'a>,
    span: Span,
    env: &mut Environment<'a>,
    writer: &mut W,
) -> Result<Value<'a>, Error<'a>> {
    let value = visit(&unary.expr, env, writer)?;

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

#[allow(clippy::result_large_err, reason = "Return is handled by error path")]
#[allow(
    clippy::string_add,
    reason = "Do not want to modify the original string inplace"
)]
fn visit_binary<'a, W: Write + Debug>(
    binary: &Binary<'a>,
    span: Span,
    env: &mut Environment<'a>,
    writer: &mut W,
) -> Result<Value<'a>, Error<'a>> {
    trace!("Binary");
    let left_value = visit(&binary.left, env, writer)?;
    let right_value = visit(&binary.right, env, writer)?;

    match binary.operator {
        BinaryOperator::EqualEqual => {
            return Ok(Value::Boolean(is_equal(&left_value, &right_value)));
        }
        BinaryOperator::BangEqual => {
            return Ok(Value::Boolean(!is_equal(&left_value, &right_value)));
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

#[allow(clippy::result_large_err, reason = "Return is handled by error path")]
fn visit_logical<'a, W: Write + Debug>(
    logical: &Logical<'a>,
    env: &mut Environment<'a>,
    writer: &mut W,
) -> Result<Value<'a>, Error<'a>> {
    let lhs_value = visit(&logical.left, env, writer)?;
    let lhs_truthy = as_bool(&lhs_value);

    match logical.operator {
        LogicalOperator::Or if !lhs_truthy => {
            visit(&logical.right, env, writer)
        }
        LogicalOperator::And if lhs_truthy => {
            visit(&logical.right, env, writer)
        }
        _ => Ok(lhs_value),
    }
}

#[allow(clippy::result_large_err, reason = "Return is handled by error path")]
fn visit_call<'a, W: Write + Debug>(
    call: &Call<'a>,
    span: Span,
    env: &mut Environment<'a>,
    writer: &mut W,
) -> Result<Value<'a>, Error<'a>> {
    trace!("Call");
    let callee = visit(&call.callee, env, writer)?;

    let function_value = match callee {
        Value::Function(func) => func,
        Value::Class(class) => class.constructor(),
        _ => {
            return Err(Error::NonFunctionCalled { span });
        }
    };

    let arguments: Vec<Value<'a>> = call
        .arguments
        .iter()
        .map(|arg| visit(arg, env, writer))
        .collect::<Result<_, _>>()?;

    call_function(function_value, arguments, span, env, writer)
}

#[allow(clippy::result_large_err, reason = "Return is handled by error path")]
fn call_function<'a, W: Write + Debug>(
    function_value: FunctionValue<'a>,
    arguments: Vec<Value<'a>>,
    span: Span,
    env: &mut Environment<'a>,
    writer: &mut W,
) -> Result<Value<'a>, Error<'a>> {
    let func = function_value.function;
    let closure = &function_value.closure;

    if arguments.len() != func.params.len() {
        return Err(Error::IncorrectArgumentCount {
            span,
            expected_arguments: func.params.len(),
            received_arguments_count: arguments.len(),
        });
    }

    match func.body {
        FunctionKind::Rust(function) => Ok(function(arguments)),
        FunctionKind::Constructor(class) => {
            let superclass = if let Some(ref binding) = class.superclass {
                match env.get(binding) {
                    Ok(Value::Class(class)) => Some(class),
                    Ok(..) => {
                        return Err(Error::SuperClassMustBeClass(binding.span));
                    }
                    Err(GetError::Undefined) => {
                        return Err(UndefinedVariable(binding.clone()));
                    }
                    Err(GetError::Uninitalised) => {
                        return Err(UninitialisedVariable(binding.clone()));
                    }
                }
            } else {
                None
            };

            let instance =
                Rc::new(RefCell::new(Instance::new(class, &superclass)));
            let Some(Value::Function(init)) =
                instance.borrow_mut().get("init").cloned()
            else {
                return Ok(Value::Instance(instance));
            };

            call_function(
                init.bind(Rc::clone(&instance)),
                arguments,
                span,
                env,
                writer,
            )?;
            Ok(Value::Instance(instance))
        }
        FunctionKind::Lox(statement) => {
            env.push(closure.clone());
            env.narrow();

            for (name, value) in func.params.iter().zip(arguments) {
                env.define(name, Some(value));
            }
            let result = eval(&statement, env, writer);
            env.pop();
            env.pop();

            match result {
                Ok(_) => Ok(Value::Nil),
                Err(Return { value, .. }) => Ok(value),
                Err(e) => Err(e),
            }
        }
        FunctionKind::CustomConstructor(statement) => {
            env.push(closure.clone());
            env.narrow();

            for (name, value) in func.params.iter().zip(arguments) {
                env.define(name, Some(value));
            }
            let eval_result = eval(&statement, env, writer);

            env.pop();
            let instance = env
                .get_local("this")
                .expect("Constructor guarantees instance");
            env.pop();

            match eval_result {
                Ok(_) | Err(Return { .. }) => Ok(instance),
                Err(e) => Err(e),
            }
        }
    }
}

#[allow(clippy::result_large_err, reason = "Return is handled by error path")]
fn visit_get<'a, W: Write + Debug>(
    get: &Get<'a>,
    env: &mut Environment<'a>,
    writer: &mut W,
) -> Result<Value<'a>, Error<'a>> {
    trace!("Get");
    let target = get.target.clone();

    let Value::Instance(instance) = visit(&get.object, env, writer)? else {
        return Err(Error::NoninstancePropertyAccess {
            object: get.object.span,
            attribute: get.target.span,
        });
    };

    let attribute = instance.borrow().get(target.name).cloned();

    match attribute {
        Some(Value::Function(method)) => {
            Ok(Value::Function(method.bind(instance)))
        }
        Some(val) => Ok(val.clone()),
        None => Err(Error::UndefinedProperty { instance, target }),
    }
}

#[allow(clippy::result_large_err, reason = "Return is handled by error path")]
fn visit_set<'a, W: Write + Debug>(
    set: &Set<'a>,
    env: &mut Environment<'a>,
    writer: &mut W,
) -> Result<Value<'a>, Error<'a>> {
    trace!("Set");
    let target = set.target.clone();

    let Value::Instance(instance) = visit(&set.object, env, writer)? else {
        return Err(Error::NoninstancePropertyAccess {
            object: set.object.span,
            attribute: set.target.span,
        });
    };

    let value = visit(&set.value, env, writer)?;

    instance.borrow_mut().set(target.name, value.clone());

    Ok(value)
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
