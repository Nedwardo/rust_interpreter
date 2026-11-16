use std::cell::RefCell;
use std::rc::Rc;

use crate::error_utils::StageError;
use crate::evaluator::environment::VariableBinding;
use crate::expressions::{BinaryOperator, UnaryOperator};
use crate::expressions::{Instance, Value};
use crate::token::Span;

const STAGE: &str = "Evaluation";

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
    UndefinedVariable(VariableBinding<'a>),
    UninitialisedVariable(VariableBinding<'a>),
    GroupErrors(Vec<Self>),
    NonFunctionCalled {
        span: Span,
    },
    IncorrectArgumentCount {
        span: Span,
        expected_arguments: usize,
        received_arguments_count: usize,
    },
    NoninstancePropertyAccess {
        object: Span,
        attribute: Span,
    },
    UndefinedProperty {
        instance: Rc<RefCell<Instance<'a>>>,
        target: VariableBinding<'a>,
    },
    SuperClassMustBeClass(Span),
    Return {
        span: Span,
        value: Value<'a>,
    },
    Break,
}

impl EvaluationError<'_> {
    const fn span(&self) -> Option<Span> {
        match self {
            Self::UnsupportedBinaryOperand { span, .. }
            | Self::UnsupportedUnaryOperand { span, .. }
            | Self::UndefinedVariable(VariableBinding { span, .. })
            | Self::UninitialisedVariable(VariableBinding { span, .. })
            | Self::Return { span, .. }
            | Self::NonFunctionCalled { span }
            | Self::IncorrectArgumentCount { span, .. }
            | Self::SuperClassMustBeClass(span) => Some(*span),
            Self::NoninstancePropertyAccess { object, attribute } => {
                Some((object.0, attribute.1))
            }
            Self::UndefinedProperty { target, .. } => {
                // Include the '.'
                Some((target.span.0 - 1, target.span.1))
            }
            Self::Break | Self::GroupErrors(..) => None,
        }
    }

    fn message(&self) -> String {
        match self {
            Self::UnsupportedBinaryOperand {
                operator,
                lhs_type,
                rhs_type,
                ..
            } => format!(
                "Unsupported operand type for {operator}: '{lhs_type}' and '{rhs_type}'"
            ),
            Self::UnsupportedUnaryOperand {
                operator,
                expr_type,
                ..
            } => {
                format!("Bad operand type for unary {operator}: '{expr_type}'")
            }
            Self::UndefinedVariable(VariableBinding { name, .. }) => {
                format!("Variable '{name}' is not defined")
            }
            Self::UninitialisedVariable(VariableBinding { name, .. }) => {
                format!("Variable '{name}' is not initialised")
            }
            Self::Return { .. } => "Return used outside of function".to_owned(),
            Self::NonFunctionCalled { .. } => {
                "Can only call functions and classes".to_owned()
            }
            Self::IncorrectArgumentCount {
                expected_arguments,
                received_arguments_count,
                ..
            } => format!(
                "Expected {expected_arguments} arguments, but got {received_arguments_count}."
            ),
            Self::NoninstancePropertyAccess { .. } => {
                "Only instances have properties".to_owned()
            }
            Self::UndefinedProperty { instance, target } => {
                format!(
                    "{} object has no attribute {}",
                    instance.borrow().class.binding.name,
                    target.name
                )
            }
            Self::SuperClassMustBeClass(..) => {
                "Superclass must be a class".to_owned()
            }
            Self::Break | Self::GroupErrors(..) => String::new(),
        }
    }

    fn children(&self) -> Vec<Self> {
        if let Self::GroupErrors(errors) = self {
            errors.clone()
        } else {
            Vec::new()
        }
    }
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
        if matches!(val, EvaluationError::Break) {
            unreachable!(
                "Parser should prevent break from being returned as an error"
            );
        }
        Self {
            span: val.span(),
            message: val.message(),
            stage: STAGE,
            children: val.children().into_iter().map(Into::into).collect(),
        }
    }
}
