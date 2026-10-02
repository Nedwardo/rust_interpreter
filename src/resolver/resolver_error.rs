use crate::{
    error_utils::StageError, evaluator::environment::VariableBinding,
    token::Span,
};

#[derive(Clone)]
pub enum ResolverError<'a> {
    VariableReferencedInInitalisation { name: String, span: Span },
    VariableAlreadyExists { name: String, span: Span },
    ReturnFromTopLevel(Span),
    Group(Vec<Self>),
    UnusedValue(VariableBinding<'a>),
}

impl<'a> ResolverError<'a> {
    pub fn unused_values(values: &Vec<VariableBinding<'a>>) -> Self {
        Self::Group(
            values
                .iter()
                .map(|v| Self::UnusedValue(v.clone()))
                .collect(),
        )
    }

    const fn span(&self) -> Option<Span> {
        match self {
            Self::VariableAlreadyExists { span, .. }
            | Self::VariableReferencedInInitalisation { span, .. }
            | Self::ReturnFromTopLevel(span)
            | Self::UnusedValue(VariableBinding { span, .. }) => Some(*span),
            Self::Group(..) => None,
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
            Self::Group(..) => String::new(),
            Self::UnusedValue(VariableBinding { name, .. }) => {
                format!("Value {name} is unused")
            }
        }
    }

    pub fn children(&self) -> Vec<Self> {
        if let Self::Group(children) = self {
            children.clone()
        } else {
            Vec::new()
        }
    }
}

impl From<ResolverError<'_>> for StageError {
    fn from(value: ResolverError) -> Self {
        Self {
            span: value.span(),
            message: value.message(),
            stage: "Resolving",
            children: value.children().into_iter().map(Into::into).collect(),
        }
    }
}
