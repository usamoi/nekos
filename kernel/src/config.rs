use crate::prelude::*;
use common::basic::{env_cast, env_match};

// kernel
pub static KERNEL_ADDRESS: PAddr = PAddr::new(env_cast!("kernel_address"));

// logging
pub const LOGGING_LEVEL: log::Level = env_match! {
    match env "logging_level" {
        "trace" => log::Level::Trace,
        "debug" => log::Level::Debug,
        "info" => log::Level::Info,
        "warn" => log::Level::Warn,
        "error" => log::Level::Error,
    }
};

// backtrace

pub const BACKTRACE_LIMIT: usize = env_cast!("backtrace_limit");

// memory
pub const FALLBACK_HEAP_SIZE: usize = 16 * 1024 * 1024;
pub const STACK_SIZE: usize = 2 * 1024 * 1024;
pub const FAULT_STACK_SIZE: usize = 8 * 1024;

// process
pub const PROCESS_INITPROC: usize = 0;
pub const PROCESS_RESERVE_HANDLES: usize = 65536;
pub const THREAD_STACK_LAYOUT: MapLayout = MapLayout::new(16 * 1024, 4096).unwrap();
