mod expressions;
mod parser;
mod read_file_error;
mod scanner;
mod token;
mod token_type;
use expressions::printers::ast_print;
use parser::Parser;
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
        [_] => run_prompt(),
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

fn run_prompt() -> Result<(), Box<dyn Error>> {
    let mut buffer = stdout();
    let mut line: String;
    loop {
        line = String::new();
        print!("> ");
        buffer
            .flush()
            .expect("I'm not sure how you could error this, good job");
        match stdin().read_line(&mut line) {
            Err(_) => break,
            Ok(_) => run(&line)?,
        }
    }
    Ok(())
}

fn run(file: &str) -> Result<(), Box<dyn Error>> {
    let mut scanner = Scanner::new(file);
    let tokens = scanner.scan_tokens().map_err(Box::new)?;

    let mut parser = Parser::new(tokens);
    let expression = parser.parse();

    if let Ok(expr) = expression {
        ast_print(&expr);
    }

    Ok(())
}
