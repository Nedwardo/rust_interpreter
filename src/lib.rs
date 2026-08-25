mod error_utils;
use log::trace;
use std::fmt::Debug;
mod evaluator;
pub mod expressions;
pub mod logger;
mod parser;
mod read_file_error;
mod scanner;
mod token;
use crate::error_utils::HydratedStageError;
use crate::expressions::Value;
use crate::parser::parse;
mod token_type;
use crate::evaluator::evaluate;
use read_file_error::ReadFileError;
use scanner::scan;
use std::error::Error;
use std::fmt;
use std::fs::read_to_string;
use std::path::Path;

/// # Errors
///
/// Will err if the program errors, or if the file is invalid
pub fn run_file<W: fmt::Write + Debug>(
    script_address: &str,
    writer: &mut W,
) -> Result<(), Box<dyn Error>> {
    let script_path = Path::new(script_address);
    let file_contents =
        read_to_string(script_path).map_err(|e| ReadFileError {
            path: script_path.into(),
            source: e,
        })?;
    run(&file_contents, writer)?;
    Ok(())
}

/// # Errors
///
/// Will err if the script errors
pub fn run<'a, W: fmt::Write + Debug>(
    script: &'a str,
    writer: &mut W,
) -> Result<Option<Value<'a>>, Box<dyn Error>> {
    let tokens = scan(script)
        .map_err(|err| HydratedStageError::hydrate_errors(err, script))?;
    let statements = parse(tokens)
        .map_err(|err| HydratedStageError::hydrate_error(&err, script))?;
    trace!("Statments: {statements:#?}");
    Ok(evaluate(&statements, writer).map_err(|err| {
        Box::new(HydratedStageError::hydrate_errors(err, script))
    })?)
}
