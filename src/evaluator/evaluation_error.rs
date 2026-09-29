use crate::error_utils::StageError;
use crate::expressions::Value;
use crate::expressions::{BinaryOperator, UnaryOperator};
use crate::token::Span;

#[derive(Debug, Clone)]
pub enum EvaluationError<'a> {
    UnsupportedBinaryOperand {
        lhs_type: &'static str,
        operator: BinaryOperator,
        rhs_type: &'static str,
        span: Span,
    },
    UnsupportedUnaryOperand {
        operator: UnaryOperator,
        expr_type: &'static str,
        span: Span,
    },
    UndefinedVariable {
        name: &'a str,
        span: Span,
    },
    UnitialisedVariable {
        name: &'a str,
        span: Span,
    },
    GroupErrors(Vec<Self>),
    NonFunctionCalled {
        span: Span,
    },
    IncorrectArgumentCount {
        span: Span,
        expected_arguments: usize,
        recieved_arguments_count: usize,
    },
    Return {
        span: Span,
        value: Value<'a>,
    },
    Break,
}

impl From<Vec<Self>> for EvaluationError<'_> {
    fn from(value: Vec<Self>) -> Self {
        if value.len() == 1
            && matches!(
                value[0],
                EvaluationError::Break | EvaluationError::Return { .. }
            )
        {
            value[0].clone()
        } else {
            EvaluationError::GroupErrors(value)
        }
    }
}

impl<'a> From<EvaluationError<'a>> for StageError {
    fn from(val: EvaluationError<'a>) -> Self {
        match val {
            EvaluationError::UnsupportedBinaryOperand {
                span,
                operator,
                lhs_type,
                rhs_type,
            } => Self {
                span: Some(span),
                message: format!(
                    "Unsupported operand type for {operator}: '{lhs_type}' and '{rhs_type}'"
                ),
                stage: "evaulation",
                children: Vec::new(),
            },
            EvaluationError::UnsupportedUnaryOperand {
                span,
                operator,
                expr_type,
            } => Self {
                span: Some(span),
                message: format!(
                    "Bad operand type for unary {operator}: '{expr_type}'"
                ),
                stage: "evaulation",
                children: Vec::new(),
            },
            EvaluationError::UndefinedVariable { name, span } => Self {
                span: Some(span),
                message: format!("Variable {name} is not defined"),
                stage: "evaulation",
                children: Vec::new(),
            },
            EvaluationError::UnitialisedVariable { name, span } => Self {
                span: Some(span),
                message: format!("Variable {name} is not initalised"),
                stage: "evaulation",
                children: Vec::new(),
            },
            EvaluationError::GroupErrors(errors) => Self {
                span: None,
                message: "Error while evaluating group".to_owned(),
                stage: "evaluation",
                children: errors.into_iter().map(Into::into).collect(),
            },
            EvaluationError::Break => unreachable!(
                "Parser should prevent break from being returned as an error"
            ),
            EvaluationError::Return { span, value: _ } => Self {
                span: Some(span),
                message: "Return used outside of function".to_owned(),
                stage: "evaluation",
                children: Vec::new(),
            },
            EvaluationError::NonFunctionCalled { span } => Self {
                span: Some(span),
                message: "Can only call functions and classes".to_owned(),
                stage: "evaluation",
                children: Vec::new(),
            },
            EvaluationError::IncorrectArgumentCount {
                span,
                expected_arguments,
                recieved_arguments_count,
            } => Self {
                span: Some(span),
                message: format!(
                    "Expected {expected_arguments} arguments, but got {recieved_arguments_count}."
                ),
                stage: "evaluation",
                children: Vec::new(),
            },
        }
    }
}
