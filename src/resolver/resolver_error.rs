use crate::{error_utils::StageError, token::Span};

pub enum ResolverError {
    VariableReferencedInInitalisation { name: String, span: Span },
    VariableAlreadyExists { name: String, span: Span },
    ReturnFromTopLevel(Span),
}

impl ResolverError {
    const fn span(&self) -> Span {
        match self {
            Self::VariableAlreadyExists { span, .. }
            | Self::VariableReferencedInInitalisation { span, .. }
            | Self::ReturnFromTopLevel(span) => *span,
        }
    }

    pub fn message(&self) -> String {
        match self {
            Self::VariableAlreadyExists { name, .. } => {
                format!("Variable {name} already exists")
            }
            Self::VariableReferencedInInitalisation { name, .. } => {
                format!("Variable {name} referenced during initalisation")
            }
            Self::ReturnFromTopLevel(..) => {
                "Can't return from top-level code".to_owned()
            }
        }
    }
}

impl From<ResolverError> for StageError {
    fn from(value: ResolverError) -> Self {
        Self {
            span: Some(value.span()),
            message: value.message(),
            stage: "Resolving",
            children: Vec::new(),
        }
    }
}
