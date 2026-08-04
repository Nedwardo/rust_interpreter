pub mod environment;
use log::trace;
pub mod evaluation_error;
mod globals;
use std::fmt::Write;
use std::rc::Rc;

use crate::evaluator::environment::{Environment, GetError};
use crate::evaluator::evaluation_error::EvaluationError;
use crate::evaluator::evaluation_error::EvaluationError::{
    Break, Return, UndefinedVariable, UnitialisedVariable,
    UnsupportedBinaryOperand, UnsupportedUnaryOperand,
};

use crate::expressions::{
    Binary, BinaryOperator, Call, Expr, ExprKind, Logical, LogicalOperator,
    Unary, UnaryOperator, Value,
};
use crate::expressions::{Function, FunctionKind, Statement};

pub fn evaluate<'a, W: Write>(
    statements: &Vec<Statement<'a>>,
    writer: &mut W,
) -> Result<Option<Value<'a>>, Vec<EvaluationError<'a>>> {
    trace!("Begining eval");
    evaluate_statements(statements, &mut Environment::new(), writer)
}

fn evaluate_statements<'a, W: Write>(
    statements: &Vec<Statement<'a>>,
    env: &mut Environment<'a>,
    writer: &mut W,
) -> Result<Option<Value<'a>>, Vec<EvaluationError<'a>>> {
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

fn eval<'a, W: Write>(
    statement: &Statement<'a>,
    env: &mut Environment<'a>,
    writer: &mut W,
) -> Result<Option<Value<'a>>, EvaluationError<'a>> {
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
        Statement::Declaration { name, expression } => {
            trace!("Declaration");
            if let Some(expr) = expression {
                let value = Some(visit(expr, env, writer)?);
                env.define(name, value);
            } else {
                env.define(name, None);
            }
        }
        Statement::FunctionDeclaration(declaration) => {
            env.define(
                declaration.name,
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
        Statement::Break => return Err(Break),
        Statement::Return {
            line,
            value: value_expr,
        } => {
            let return_value = match value_expr {
                Some(expr) => visit(expr, env, writer)?,
                None => Value::Nil,
            };

            return Err(Return {
                line: *line,
                value: return_value,
            });
        }
    }
    Ok(None)
}

fn define_function<'a>(
    declaration: &Function<'a>,
    env: &Environment<'a>,
) -> Value<'a> {
    Value::Function {
        declaration: declaration.clone(),
        closure: Rc::clone(env.top()),
    }
}

fn visit<'a, W: Write>(
    expr: &Expr<'a>,
    env: &mut Environment<'a>,
    writer: &mut W,
) -> Result<Value<'a>, EvaluationError<'a>> {
    match &expr.kind {
        ExprKind::Literal(value) => Ok(value.clone()),
        ExprKind::Unary(unary) => visit_unary(unary, expr.line, env, writer),
        ExprKind::Binary(binary) => {
            visit_binary(binary, expr.line, env, writer)
        }
        ExprKind::Grouping(expr) => visit(expr, env, writer),
        ExprKind::Identifier(name) => env.get(name).map_err(|err| match err {
            GetError::Undefined => UndefinedVariable {
                name,
                line: expr.line,
            },
            GetError::Uninitalised => UnitialisedVariable {
                name,
                line: expr.line,
            },
        }),
        ExprKind::Assignment(assignment) => {
            let value = visit(&assignment.expr, env, writer)?;
            env.update(assignment.name, value.clone()).map_err(|()| {
                UndefinedVariable {
                    name: assignment.name,
                    line: expr.line,
                }
            })?;
            Ok(value)
        }
        ExprKind::Logical(logical) => visit_logical(logical, env, writer),
        ExprKind::Call(call) => visit_call(call, expr.line, env, writer),
        ExprKind::Lambda(function) => Ok(define_function(function, env)),
    }
}

fn visit_unary<'a, W: Write>(
    unary: &Unary<'a>,
    line: usize,
    env: &mut Environment<'a>,
    writer: &mut W,
) -> Result<Value<'a>, EvaluationError<'a>> {
    let value = visit(&unary.expr, env, writer)?;

    match unary.operator {
        UnaryOperator::MINUS => match value {
            Value::Number(num) => Ok(Value::Number(-num)),
            _ => Err(UnsupportedUnaryOperand {
                expr_type: value.type_name(),
                operator: unary.operator,
                line,
            }),
        },
        UnaryOperator::BANG => Ok(Value::Boolean(!as_bool(&value))),
    }
}

#[allow(
    clippy::string_add,
    reason = "Do not want to modify the original string inplace"
)]
fn visit_binary<'a, W: Write>(
    binary: &Binary<'a>,
    line: usize,
    env: &mut Environment<'a>,
    writer: &mut W,
) -> Result<Value<'a>, EvaluationError<'a>> {
    let left_value = visit(&binary.left, env, writer)?;
    let right_value = visit(&binary.right, env, writer)?;

    match binary.operator {
        BinaryOperator::EQUAL_EQUAL => {
            return Ok(Value::Boolean(is_equal(&left_value, &right_value)));
        }
        BinaryOperator::BANG_EQUAL => {
            return Ok(Value::Boolean(!is_equal(&left_value, &right_value)));
        }
        BinaryOperator::PLUS => {
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
        line,
    })
}

fn visit_logical<'a, W: Write>(
    logical: &Logical<'a>,
    env: &mut Environment<'a>,
    writer: &mut W,
) -> Result<Value<'a>, EvaluationError<'a>> {
    let lhs_value = visit(&logical.left, env, writer)?;
    let lhs_truthy = as_bool(&lhs_value);

    match logical.operator {
        LogicalOperator::OR if !lhs_truthy => {
            visit(&logical.right, env, writer)
        }
        LogicalOperator::AND if lhs_truthy => {
            visit(&logical.right, env, writer)
        }
        _ => Ok(lhs_value),
    }
}

fn visit_call<'a, W: Write>(
    call: &Call<'a>,
    line: usize,
    env: &mut Environment<'a>,
    writer: &mut W,
) -> Result<Value<'a>, EvaluationError<'a>> {
    trace!("Call");
    let function = visit(&call.callee, env, writer)?;

    let Value::Function {
        declaration:
            Function {
                body,
                params,
                name: _,
            },
        closure,
    } = function
    else {
        return Err(EvaluationError::NonFunctionCalled { line });
    };
    if call.arguments.len() != params.len() {
        return Err(EvaluationError::IncorrectArgumentCount {
            line,
            expected_arguments: params.len(),
            recieved_arguments_count: call.arguments.len(),
        });
    }

    let arguments: Vec<Value<'a>> = call
        .arguments
        .iter()
        .map(|arg| visit(arg, env, writer))
        .collect::<Result<_, _>>()?;

    match body {
        FunctionKind::Rust(function) => Ok(function(arguments)),
        FunctionKind::Lox(statement) => {
            env.push(closure);
            env.narrow();

            for index in 0..params.len() {
                env.define(params[index], Some(arguments[index].clone()));
            }
            trace!("Calling with {arguments:?}");
            let result = eval(&statement, env, writer);
            env.pop();
            env.pop();

            match result {
                Ok(_) => Ok(Value::Nil),
                Err(Return { line: _, value }) => {
                    trace!("Returning value: {value}");
                    Ok(value)
                }
                Err(e) => Err(e),
            }
        }
    }
}

#[allow(clippy::float_cmp, reason = "User is trying to float cmp")]
fn numeric_binary_operations<'a>(
    lhs: f64,
    operator: BinaryOperator,
    rhs: f64,
) -> Option<Value<'a>> {
    let result = match operator {
        BinaryOperator::MINUS => Value::Number(lhs - rhs),
        BinaryOperator::SLASH => Value::Number(lhs / rhs),
        BinaryOperator::STAR => Value::Number(lhs * rhs),
        BinaryOperator::PLUS => {
            trace!("Summation of {lhs} and {rhs}");
            Value::Number(lhs + rhs)
        }
        BinaryOperator::GREATER => Value::Boolean(lhs > rhs),
        BinaryOperator::GREATER_EQUAL => Value::Boolean(lhs >= rhs),
        BinaryOperator::LESS => Value::Boolean(lhs < rhs),
        BinaryOperator::LESS_EQUAL => Value::Boolean(lhs <= rhs),
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
        Value::Function { .. } => return false,
    }
    false
}
