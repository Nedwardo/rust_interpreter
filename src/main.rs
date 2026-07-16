mod error_utils;
use log::trace;
mod evaluator;
mod expressions;
mod logger;
mod parser;
mod read_file_error;
mod scanner;
mod token;
use crate::error_utils::FlattenedError;
use crate::parser::parse;
use std::fmt::Display;
mod token_type;
use crate::evaluator::evaluate;
use crate::logger::init as logger_init;
use log::LevelFilter;
use read_file_error::ReadFileError;
use scanner::scan;
use std::env::args;
use std::error::Error;
use std::fs::read_to_string;
use std::io::{Write as _, stdin, stdout};
use std::path::Path;

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = args().collect();

    logger_init(LevelFilter::Trace)?;

    match args.as_slice() {
        [_] => run_prompt(),
        [_, file] => run_file(file),
        _ => Err("Usage: jlox [script]".into()),
    }
}

fn run_file(script_address: &str) -> Result<(), Box<dyn Error>> {
    let script_path = Path::new(script_address);
    let file_contents =
        read_to_string(script_path).map_err(|e| ReadFileError {
            path: script_path.into(),
            source: e,
        })?;
    run(&file_contents)?;
    Ok(())
}

#[allow(clippy::print_stderr, reason = "cli app")]
fn run_prompt() -> Result<(), Box<dyn Error>> {
    let mut buffer = stdout();
    let mut line: String;
    loop {
        line = String::new();
        print!("> ");
        buffer.flush()?;
        let _ = stdin().read_line(&mut line)?;

        line.truncate(line.len() - 1);
        match run(&line) {
            Ok(Some(result)) => println!("{result}\n"),
            Err(e) => eprintln!("{e}"),
            Ok(_) => {}
        }
    }
}

fn run(file: &str) -> Result<Option<impl Display + use<'_>>, Box<dyn Error>> {
    let tokens = scan(file).map_err(Box::new)?;
    let statements = parse(tokens)
        .map_err(|err| Box::new(FlattenedError::flatten(err, file)))?;
    trace!("{statements:#?}");
    Ok(evaluate(&statements)
        .map_err(|err| Box::new(FlattenedError::flatten(err, file)))?)
}
