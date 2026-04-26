mod expressions;
// mod interpreter;
mod parser;
mod read_file_error;
mod scanner;
mod token;
mod token_type;
use expressions::format_ast;
use parser::parse;
use read_file_error::ReadFileError;
use scanner::Scanner;
use std::env::args;
use std::error::Error;
use std::fs::read_to_string;
use std::io::{Write as _, stdin, stdout};
use std::path::Path;

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = args().collect();

    match args.as_slice() {
        [_] => {
            run_prompt();
            Ok(())
        }
        [_, file] => run_file(file),
        _ => Err("Usage: jlox [script]".into()),
    }?;
    Ok(())
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
fn run_prompt() {
    let mut buffer = stdout();
    let mut line: String;
    loop {
        line = String::new();
        print!("> ");
        buffer
            .flush()
            .expect("I'm not sure how you could error this, good job");

        if stdin().read_line(&mut line).is_err() {
            break;
        }

        line.truncate(line.len() - 1);
        if let Err(err) = run(&line) {
            eprintln!("{err}");
        }
    }
}

fn run(file: &str) -> Result<(), Box<dyn Error>> {
    let mut scanner = Scanner::new(file);
    let tokens = scanner.scan_tokens().map_err(Box::new)?;

    let expression = parse(tokens);

    match expression {
        Ok(expr) => println!("{}", format_ast(&expr)),
        Err(err) => eprintln!("Failed to parse {err:?}"),
    }

    Ok(())
}
