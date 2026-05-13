pub mod interpreter_error;

use crate::interpreter::interpreter_error::InterpreterError;
use crate::interpreter::interpreter_error::InterpreterError::{
    UnsupportedBinaryOperand, UnsupportedUnaryOperand,
};

use crate::expressions::Statment;
use crate::expressions::{Binary, ExprKind, Unary, Valua};
use crate::expressions::{BinaryOperator, Expr, UnaryOperator};

pub fn interpret(
    statments: &Vec<Statment<'_>>,
) -> Result<Box<Valua>, InterpreterError> {
    for statment in statments {
        visit(expr);
    }
}

fn visit<'a>(expr: &'a Expr<'a>) -> Result<Box<Valua>, InterpreterError> {
    match &expr.kind {
        ExprKind::Literal(value) => Ok(Box::new(value.clone())),
        ExprKind::Identifier(name) => get_identifier(name),
        ExprKind::Unary(unary) => visit_unary(unary, expr.line),
        ExprKind::Binary(binary) => visit_binary(binary, expr.line),
        ExprKind::Grouping(expr) => visit(expr),
    }
}

fn get_identifier(name: &'_ str) -> Result<Box<Valua>, InterpreterError> {
    todo!()
}

fn visit_unary(
    unary: &Unary,
    line: usize,
) -> Result<Box<Valua>, InterpreterError> {
    let value = visit(&unary.expr)?;

    match unary.operator {
        UnaryOperator::MINUS => match *value {
            Valua::Number(num) => Ok(Box::new(Valua::Number(-num))),
            _ => Err(UnsupportedUnaryOperand {
                expr_type: value.type_name(),
                operator: unary.operator,
                line,
            }),
        },
        UnaryOperator::BANG => Ok(Box::new(Valua::Boolean(!as_bool(&value)))),
    }
}

#[allow(
    clippy::string_add,
    reason = "Do not want to modify the original string inplace"
)]
fn visit_binary(
    binary: &Binary,
    line: usize,
) -> Result<Box<Valua>, InterpreterError> {
    let left_value = visit(&binary.left)?;
    let right_value = visit(&binary.right)?;

    match binary.operator {
        BinaryOperator::EQUAL_EQUAL => {
            return Ok(Box::new(Valua::Boolean(is_equal(
                &left_value,
                &right_value,
            ))));
        }
        BinaryOperator::BANG_EQUAL => {
            return Ok(Box::new(Valua::Boolean(!is_equal(
                &left_value,
                &right_value,
            ))));
        }
        BinaryOperator::PLUS => {
            if let Valua::String(lhs_string) = *left_value {
                return Ok(Box::new(Valua::String(
                    lhs_string + &right_value.cast_to_string(),
                )));
            } else if let Valua::String(rhs_string) = *right_value {
                return Ok(Box::new(Valua::String(
                    left_value.cast_to_string() + &rhs_string,
                )));
            }
        }
        _ => {}
    }

    let left_type = left_value.type_name();

    if let Valua::Number(lhs) = *left_value
        && let Valua::Number(rhs) = *right_value
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
) -> Option<Valua> {
    let result = match operator {
        BinaryOperator::MINUS => Valua::Number(lhs - rhs),
        BinaryOperator::SLASH => Valua::Number(lhs / rhs),
        BinaryOperator::STAR => Valua::Number(lhs * rhs),
        BinaryOperator::PLUS => Valua::Number(lhs + rhs),
        BinaryOperator::GREATER => Valua::Boolean(lhs > rhs),
        BinaryOperator::GREATER_EQUAL => Valua::Boolean(lhs >= rhs),
        BinaryOperator::LESS => Valua::Boolean(lhs < rhs),
        BinaryOperator::LESS_EQUAL => Valua::Boolean(lhs <= rhs),
        _ => {
            return None;
        }
    };
    Some(result)
}

pub const fn as_bool(value: &Valua) -> bool {
    match *value {
        Valua::Nil => false,
        Valua::Boolean(bool_value) => bool_value,
        _ => true,
    }
}

#[allow(clippy::float_cmp, reason = "User is trying to float cmp")]
fn is_equal(left_value: &Valua, right_value: &Valua) -> bool {
    match left_value {
        Valua::String(lhs) => {
            if let Valua::String(rhs) = right_value {
                return lhs == rhs;
            }
        }
        Valua::Number(lhs) => {
            if let Valua::Number(rhs) = *right_value {
                return *lhs == rhs;
            }
        }
        Valua::Boolean(lhs) => {
            if let Valua::Boolean(rhs) = *right_value {
                return *lhs == rhs;
            }
        }
        Valua::Nil => {
            return matches!(*right_value, Valua::Nil);
        }
    }
    false
}
