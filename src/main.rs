mod expressions;
mod interpreter_error;
mod keywords;
mod read_file_error;
mod scanner;
mod token;
mod token_type;
mod parser;
use read_file_error::ReadFileError;
use scanner::build_scanner;
use std::env::args;
use std::fs::read_to_string;
use std::io::{Write, stdin, stdout};
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = args().collect();

    match &args[..] {
        [_] => run_prompt(),
        [_, file] => run_file(file)?,
        _ => {
            return Err("Usage: jlox [script]".into());
        }
    }
    Ok(())
}

fn run_file(script_address: &str) -> Result<(), ReadFileError> {
    let script_path = Path::new(script_address);
    let file_contents = read_to_string(script_path).map_err(|e| ReadFileError {
        path: script_path.into(),
        source: e,
    })?;
    run(&file_contents);
    Ok(())
}

fn run_prompt() {
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
            Ok(_) => run(&line),
        }
    }
}

fn run(file: &str) {
    let mut scanner = build_scanner(file);
    let tokens = scanner.scan_tokens();

    for token in tokens.iter() {
        println!("{token}")
    }
}
