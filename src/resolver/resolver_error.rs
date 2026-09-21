use crate::error_utils::StageError;

pub enum ResolverError {
    VariableReferencedInInitalisation { name: String, line: usize },
}

impl From<ResolverError> for StageError {
    fn from(value: ResolverError) -> Self {
        match value {
            ResolverError::VariableReferencedInInitalisation { name, line } => {
                Self {
                    line: Some(line),
                    message: format!(
                        "Variable {name} referenced during initalisation"
                    ),
                    error_location: Some(name),
                    stage: "Resolving",
                    children: vec![],
                }
            }
        }
    }
}
