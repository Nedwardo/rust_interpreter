use crate::{error_utils::StageError, token::Span};

pub enum ResolverError {
    VariableReferencedInInitalisation { name: String, span: Span },
    VariableAlreadyExists { name: String, span: Span },
    ReturnFromTopLevel(Span),
}

impl From<ResolverError> for StageError {
    fn from(value: ResolverError) -> Self {
        match value {
            ResolverError::VariableReferencedInInitalisation { name, span } => {
                Self {
                    span: Some(span),
                    message: format!(
                        "Variable {name} referenced during initalisation"
                    ),
                    stage: "Resolving",
                    children: vec![],
                }
            }
            ResolverError::VariableAlreadyExists { name, span } => Self {
                span: Some(span),
                message: format!("Variable {name} already exists"),
                stage: "Resolving",
                children: vec![],
            },
            ResolverError::ReturnFromTopLevel(span) => Self {
                span: Some(span),
                message: "Can't return from top-level code".to_owned(),
                stage: "Resolving",
                children: vec![],
            },
        }
    }
}
