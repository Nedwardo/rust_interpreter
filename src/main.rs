use interpreter::{run, run_file};
use std::env::args;
use std::error::Error;
use std::fmt;
use std::io;
use std::io::{Write as _, stdin, stdout};

struct IoWriteAdapter<W>(pub W);

#[allow(
    clippy::map_err_ignore,
    reason = "Type narrowing to marry std::io and std::fmt for testing and cli app"
)]
impl<W: io::Write> fmt::Write for IoWriteAdapter<W> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.0.write_all(s.as_bytes()).map_err(|_| fmt::Error)
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = args().collect();

    match args.as_slice() {
        [_] => run_prompt(),
        [_, file] => run_file(file, &mut IoWriteAdapter(stdout())),
        _ => Err("Usage: jlox [script]".into()),
    }
}

#[allow(clippy::print_stderr, reason = "cli app")]
fn run_prompt() -> Result<(), Box<dyn Error>> {
    let mut buffer = IoWriteAdapter(stdout());
    let mut line: String;
    loop {
        line = String::new();
        print!("> ");
        buffer.0.flush()?;
        let _ = stdin().read_line(&mut line)?;

        line.truncate(line.len() - 1);
        match run(&line, &mut buffer) {
            Ok(Some(result)) => println!("{result}\n"),
            Err(e) => eprintln!("{e}"),
            Ok(_) => {}
        }
    }
}
