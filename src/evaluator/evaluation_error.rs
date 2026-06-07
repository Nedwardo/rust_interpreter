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

impl<'a> From<EvaluationError<'a>> for StageError {
    fn from(val: EvaluationError<'a>) -> Self {
        match val {
            EvaluationError::UnsupportedBinaryOperand {
                line,
                operator,
                lhs_type,
                rhs_type,
            } => Self {
                line: Some(line),
                message: format!(
                    "Line {line}\nEvaluation Error: Unsupported operand type for {operator}: '{lhs_type}' and '{rhs_type}'"
                ),
                error_location: None,
                stage: "parsing",
                children: Vec::new(),
            },
            EvaluationError::UnsupportedUnaryOperand {
                line,
                operator,
                expr_type,
            } => Self {
                line: Some(line),
                message: format!(
                    "Line {line}\n Evaluation Error: Bad operand type for unary {operator}: '{expr_type}'"
                ),
                error_location: None,
                stage: "parsing",
                children: Vec::new(),
            },
            EvaluationError::UndefinedVariable { name, line } => Self {
                line: Some(line),
                message: format!(
                    "Line {line}\n Evaluation Error: Variable {name} is not defined"
                ),
                error_location: Some(name.to_owned()),
                stage: "parsing",
                children: Vec::new(),
            },
        }
    }
}
