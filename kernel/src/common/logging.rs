use crate::prelude::*;
use arch::cpu::checked_local;
use arch::stdout::STDOUT;
use arch::time::MachineInstant;
use core::fmt::Write;
use log::{Level, LevelFilter, Log, Metadata, Record};
use owo_colors::OwoColorize;

pub struct Logger;

impl Log for Logger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= config::LOGGING_LEVEL
    }

    fn log(&self, record: &Record) {
        if !self.enabled(record.metadata()) {
            return;
        }
        use Level::*;
        let s = &mut *STDOUT.write.lock();
        match record.level() {
            Error => write!(s, "{}", "Error".red()).unwrap(),
            Warn => write!(s, "{}", "Warn".yellow()).unwrap(),
            Info => write!(s, "{}", "Info".green()).unwrap(),
            Debug => write!(s, "{}", "Debug".blue()).unwrap(),
            Trace => write!(s, "{}", "Trace".cyan()).unwrap(),
        }
        if let Some(ms) = MachineInstant::now()
            .checked_duration_since(MachineInstant::ZERO)
            .map(|x| x.as_millis())
        {
            write!(s, " [{:#2}.{:#03}]", ms / 1000, ms % 1000).unwrap();
        }
        if let Some(id) = checked_local().and_then(|local| local.get_id()) {
            write!(s, " [CPU {}]", id).unwrap();
        }
        writeln!(s, " {}", record.args()).unwrap();
    }

    fn flush(&self) {}
}

pub static LOGGER: Logger = Logger;

pub unsafe fn init_boot() {
    log::set_logger(&LOGGER).unwrap();
    log::set_max_level(LevelFilter::Trace);
}
