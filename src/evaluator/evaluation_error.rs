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
    UnitialisedVariable {
        name: &'a str,
        line: usize,
    },
    GroupErrors(Vec<Self>),
    NonFunctionCalled {
        line: usize,
    },
    IncorrectArgumentCount {
        line: usize,
        expected_arguments: usize,
        recieved_arguments_count: usize,
    },
    Break,
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
                stage: "evaulation",
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
                stage: "evaulation",
                children: Vec::new(),
            },
            EvaluationError::UndefinedVariable { name, line } => Self {
                line: Some(line),
                message: format!(
                    "Line {line}\n Evaluation Error: Variable {name} is not defined"
                ),
                error_location: Some(name.to_owned()),
                stage: "evaulation",
                children: Vec::new(),
            },
            EvaluationError::UnitialisedVariable { name, line } => Self {
                line: Some(line),
                message: format!(
                    "Line {line}\n Evaluation Error: Variable {name} is not initalised"
                ),
                error_location: Some(name.to_owned()),
                stage: "evaulation",
                children: Vec::new(),
            },
            EvaluationError::GroupErrors(errors) => Self {
                line: None,
                message: "Error while parsing group".to_owned(),
                error_location: None,
                stage: "parsing",
                children: errors.into_iter().map(Into::into).collect(),
            },
            EvaluationError::Break => unreachable!(
                "Parser should prevent break from being returned as an error"
            ),
            EvaluationError::NonFunctionCalled { line } => Self {
                line: Some(line),
                message: "Can only call functions and classes".to_owned(),
                error_location: None,
                stage: "evaluation",
                children: Vec::new(),
            },
            EvaluationError::IncorrectArgumentCount {
                line,
                expected_arguments,
                recieved_arguments_count,
            } => Self {
                line: Some(line),
                message: format!(
                    "Expected {expected_arguments} arguments, but got {recieved_arguments_count}."
                ),
                error_location: None,
                stage: "evaluation",
                children: Vec::new(),
            },
        }
    }
}
