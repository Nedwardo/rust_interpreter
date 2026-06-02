use crate::error_utils::StageError;

use crate::expressions::{BinaryOperator, UnaryOperator};

#[derive(Debug, Clone)]
pub enum EvaluationError<'a> {
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

impl<'a> From<EvaluationError<'a>> for StageError<'a> {
    fn from(val: EvaluationError<'a>) -> Self {
        match val {
            EvaluationError::UnsupportedBinaryOperand {
                line,
                operator,
                lhs_type,
                rhs_type,
            } => StageError {
                line: Some(line),
                message: format!(
                    "Line {line}\nEvaluation Error: Unsupported operand type for {operator}: '{lhs_type}' and '{rhs_type}'"
                ),
                error_location: None,
                stage: "parsing",
                child: None,
            },
            EvaluationError::UnsupportedUnaryOperand {
                line,
                operator,
                expr_type,
            } => StageError {
                line: Some(line),
                message: format!(
                    "Line {line}\n Evaluation Error: Bad operand type for unary {operator}: '{expr_type}'"
                ),
                error_location: None,
                stage: "parsing",
                child: None,
            },
            EvaluationError::UndefinedVariable { name, line } => StageError {
                line: Some(line),
                message: format!(
                    "Line {line}\n Evaluation Error: Variable {name} is not defined"
                ),
                error_location: Some(name),
                stage: "parsing",
                child: None,
            },
        }
    }
}
