use crate::prelude::*;
use core::fmt::Write;
use log::{Level, LevelFilter, Log, Metadata, Record};
use owo_colors::OwoColorize;
use rt::thread::current;
use rt::time::Instant;

pub struct Logger;

impl Log for Logger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= config::logging_level()
    }

    fn log(&self, record: &Record) {
        if !self.enabled(record.metadata()) {
            return;
        }
        use Level::*;
        let mut s = rt::io::stdout().lock();
        match record.level() {
            Error => write!(s, "{}", "Error".red()).unwrap(),
            Warn => write!(s, "{}", "Warn".yellow()).unwrap(),
            Info => write!(s, "{}", "Info".green()).unwrap(),
            Debug => write!(s, "{}", "Debug".blue()).unwrap(),
            Trace => write!(s, "{}", "Trace".cyan()).unwrap(),
        }
        if let Some(ms) = Instant::maybe_now()
            .map(|x| x - Instant::ZERO)
            .map(|x| x.as_millis())
        {
            write!(s, " [{:#2}.{:#03}]", ms / 1000, ms % 1000).unwrap();
        }
        write!(s, " [CPU {}]", current().id()).unwrap();
        writeln!(s, " {}", record.args()).unwrap();
    }

    fn flush(&self) {}
}

pub fn logger() -> &'static Logger {
    static LOGGER: Logger = Logger;
    &LOGGER
}

pub fn init_global() {
    log::set_logger(logger()).unwrap();
    log::set_max_level(LevelFilter::Trace);
}
