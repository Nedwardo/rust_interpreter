use std::error::Error;
use std::fmt;
use std::fmt::{Display, Formatter};

use crate::expressions::{BinaryOperator, UnaryOperator};

#[derive(Debug, Clone)]
pub enum InterpreterError<'a> {
    UnsupportedBinaryOperand {
        lhs_type: &'static str,
        operator: BinaryOperator,
        rhs_type: &'static str,
        line: usize,
    },
    UnsupportedUnaryOperand {
        operator: UnaryOperator,
        expr_type: &'static str,
        line: usize,
    },
    UndefinedVariable {
        name: &'a str,
        line: usize,
    },
}

impl<'a> Display for InterpreterError<'a> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Self::UnsupportedBinaryOperand {
                line,
                operator,
                lhs_type,
                rhs_type,
            } => write!(
                f,
                "Line {line}\nInterpreter Error: Unsupported operand type for {operator}: '{lhs_type}' and '{rhs_type}'"
            ),
            Self::UnsupportedUnaryOperand {
                operator,
                expr_type,
                line,
            } => write!(
                f,
                "Line {line}\n Interpreter Error: Bad operand type for unary {operator}: '{expr_type}'"
            ),
            Self::UndefinedVariable { name, line } => write!(
                f,
                "Line {line}\n Interpreter Error: Variable {name} is not defined"
            ),
        }
    }
}

impl<'a> Error for InterpreterError<'a> {}
