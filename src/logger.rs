use log::{LevelFilter, SetLoggerError};
use log::{Metadata, Record};

static LOGGER: SimpleLogger = SimpleLogger;
struct SimpleLogger;

impl log::Log for SimpleLogger {
    fn enabled(&self, _: &Metadata) -> bool {
        true
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            println!("{} - {}", record.level(), record.args());
        }
    }

    fn flush(&self) {}
}

/// # Errors
///
/// Will err if the logger fails to be set
pub fn init(level: LevelFilter) -> Result<(), SetLoggerError> {
    log::set_logger(&LOGGER).map(|()| log::set_max_level(level))
}
