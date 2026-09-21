use clap::Parser;
use clap_verbosity::Verbosity;
use interpreter::logger::init as logger_init;
use interpreter::{run, run_file};
use std::error::Error;
use std::fmt;
use std::io;
use std::io::{Write as _, stdin, stdout};

#[derive(Debug)]
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

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    file_path: Option<std::path::PathBuf>,

    #[command(flatten)]
    verbose: Verbosity,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    logger_init(args.verbose.log_level_filter())?;

    args.file_path.map_or_else(run_prompt, |file| {
        run_file(&file, &mut IoWriteAdapter(stdout()))
    })
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
