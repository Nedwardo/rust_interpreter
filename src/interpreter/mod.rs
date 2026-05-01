pub mod interpreter_error;

use crate::interpreter::interpreter_error::InterpreterError;
use crate::interpreter::interpreter_error::InterpreterError::{
    UnsupportedBinaryOperand, UnsupportedUnaryOperand,
};

use crate::expressions::{Binary, ExprKind, Unary, Value};
use crate::expressions::{BinaryOperator, Expr, UnaryOperator};

pub fn interpret(expr: &Expr<'_>) -> Result<Box<Value>, InterpreterError> {
    visit(expr)
}

fn visit<'a>(expr: &'a Expr<'a>) -> Result<Box<Value>, InterpreterError> {
    match &expr.kind {
        ExprKind::Literal(value) => Ok(Box::new(value.clone())),
        ExprKind::Identifier(name) => get_identifier(name),
        ExprKind::Unary(unary) => visit_unary(unary, expr.line),
        ExprKind::Binary(binary) => visit_binary(binary, expr.line),
        ExprKind::Grouping(expr) => visit(expr),
    }
}

fn get_identifier(name: &'_ str) -> Result<Box<Value>, InterpreterError> {
    todo!()
}

fn visit_unary(
    unary: &Unary,
    line: usize,
) -> Result<Box<Value>, InterpreterError> {
    let value = visit(&unary.expr)?;

    match unary.operator {
        UnaryOperator::MINUS => match *value {
            Value::Number(num) => Ok(Box::new(Value::Number(-num))),
            _ => Err(UnsupportedUnaryOperand {
                expr_type: value.type_name(),
                operator: unary.operator,
                line,
            }),
        },
        UnaryOperator::BANG => Ok(Box::new(Value::Boolean(!as_bool(&value)))),
    }
}

#[allow(
    clippy::string_add,
    reason = "Do not want to modify the original string inplace"
)]
fn visit_binary(
    binary: &Binary,
    line: usize,
) -> Result<Box<Value>, InterpreterError> {
    let left_value = visit(&binary.left)?;
    let right_value = visit(&binary.right)?;

    match binary.operator {
        BinaryOperator::EQUAL_EQUAL => {
            return Ok(Box::new(Value::Boolean(is_equal(
                &left_value,
                &right_value,
            ))));
        }
        BinaryOperator::BANG_EQUAL => {
            return Ok(Box::new(Value::Boolean(!is_equal(
                &left_value,
                &right_value,
            ))));
        }
        BinaryOperator::PLUS => {
            if let Value::String(lhs_string) = *left_value {
                return Ok(Box::new(Value::String(
                    lhs_string + &right_value.cast_to_string(),
                )));
            } else if let Value::String(rhs_string) = *right_value {
                return Ok(Box::new(Value::String(
                    left_value.cast_to_string() + &rhs_string,
                )));
            }
        }
        _ => {}
    }

    let left_type = left_value.type_name();

    if let Value::Number(lhs) = *left_value
        && let Value::Number(rhs) = *right_value
        && let Some(value) =
            numeric_binary_operations(lhs, binary.operator, rhs)
    {
        return Ok(Box::new(value));
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
