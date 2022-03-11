use crate::prelude::*;
use core::fmt;
use spin::Mutex;

pub struct StdoutWrite {}

impl fmt::Write for StdoutWrite {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for c in s.chars() {
            arch::sbi::legacy::console_putchar(c as i32);
        }
        Ok(())
    }
}

pub struct Stdout {
    pub write: Mutex<StdoutWrite>,
}

impl Stdout {
    const fn new() -> Self {
        Self {
            write: Mutex::new(StdoutWrite {}),
        }
    }
}

pub static STDOUT: Stdout = Stdout::new();
