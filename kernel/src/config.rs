use crate::prelude::*;
use core::time::Duration;

// log
pub fn logging_level() -> log::Level {
    match env!("log_level") {
        "trace" => log::Level::Trace,
        "debug" => log::Level::Debug,
        "info" => log::Level::Info,
        "warn" => log::Level::Warn,
        "error" => log::Level::Error,
        _ => panic!("unknown value of environment variable log_level"),
    }
}

// backtrace

pub const BACKTRACE: usize = 64;

// memory
pub const HEAP_SIZE: usize = 16 * 1024 * 1024;
pub const STACK_SIZE: usize = 2 * 1024 * 1024;

// process
pub const PROCESS_RESERVE_HANDLES: usize = 65536;
pub const THREAD_STACK_LAYOUT: MapLayout = MapLayout::new(16 * 1024, 4096).unwrap();

// schedule
pub const SCHEDULE_TIMESLICE: Duration = Duration::from_millis(10);
