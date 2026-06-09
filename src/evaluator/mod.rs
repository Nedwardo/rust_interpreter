mod environment;
pub mod evaluation_error;

use crate::evaluator::environment::Environment;
use crate::evaluator::evaluation_error::EvaluationError;
use crate::evaluator::evaluation_error::EvaluationError::{
    UnsupportedBinaryOperand, UnsupportedUnaryOperand,
};

use crate::error_utils::StageError;
use crate::expressions::Statement;
use crate::expressions::{Binary, ExprKind, Unary, Value};
use crate::expressions::{BinaryOperator, Expr, UnaryOperator};

pub fn evaluate<'a>(
    statments: &'a Vec<Statement<'a>>,
) -> Result<(), Vec<impl Into<StageError> + use<'a>>> {
    evaluate_statments(statments, &mut Environment::new())
}

fn evaluate_statments<'a>(
    statments: &'a Vec<Statement<'a>>,
    env: &mut Environment<'a>,
) -> Result<(), Vec<impl Into<StageError> + use<'a>>> {
    let mut errors = Vec::new();
    for statment in statments {
        match statment {
            Statement::Expression(expr) => {
                if let Err(e) = visit(expr, env) {
                    errors.push(e);
                }
            }
            Statement::Print(expr) => match visit(expr, env) {
                Ok(value) => println!("{value}"),
                Err(e) => errors.push(e),
            },
            Statement::Declaration { name, expression } => match expression {
                Some(expr) => match visit(expr, env) {
                    Ok(value) => env.define(name, Some(value)),
                    Err(e) => errors.push(e),
                },
                None => env.define(name, None),
            },
            Statement::Group(s) => {
                env.narrow();
                if let Err(e) = evaluate_statments(s, env) {
                    errors.extend(e.into_iter());
                }
                env.pop_scope();
            }
        }
    }
    if !errors.is_empty() {
        return Err(errors);
    }
    Ok(())
}

fn visit<'a>(
    expr: &'a Expr<'a>,
    env: &mut Environment<'a>,
) -> Result<Value, EvaluationError<'a>> {
    match &expr.kind {
        ExprKind::Literal(value) => Ok(value.clone()),
        ExprKind::Unary(unary) => visit_unary(unary, expr.line, env),
        ExprKind::Binary(binary) => visit_binary(binary, expr.line, env),
        ExprKind::Grouping(expr) => visit(expr, env),
        ExprKind::Identifier(name) => {
            env.get(name)
                .cloned()
                .ok_or(EvaluationError::UndefinedVariable {
                    name,
                    line: expr.line,
                })
        }
        ExprKind::Assignment(assignment) => {
            let value = visit(&assignment.expr, env)?;
            env.update(assignment.name, value.clone()).map_err(|()| {
                EvaluationError::UndefinedVariable {
                    name: assignment.name,
                    line: expr.line,
                }
            })?;
            Ok(value)
        }
    }
}

fn visit_unary<'a>(
    unary: &'a Unary,
    line: usize,
    env: &mut Environment<'a>,
) -> Result<Value, EvaluationError<'a>> {
    let value = visit(&unary.expr, env)?;

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
fn visit_binary<'a>(
    binary: &'a Binary,
    line: usize,
    env: &mut Environment<'a>,
) -> Result<Value, EvaluationError<'a>> {
    let left_value = visit(&binary.left, env)?;
    let right_value = visit(&binary.right, env)?;

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

#[allow(clippy::float_cmp, reason = "User is trying to float cmp")]
fn numeric_binary_operations(
    lhs: f64,
    operator: BinaryOperator,
    rhs: f64,
) -> Option<Value> {
    let result = match operator {
        BinaryOperator::MINUS => Value::Number(lhs - rhs),
        BinaryOperator::SLASH => Value::Number(lhs / rhs),
        BinaryOperator::STAR => Value::Number(lhs * rhs),
        BinaryOperator::PLUS => Value::Number(lhs + rhs),
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
    }
    false
}
