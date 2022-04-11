use crate::prelude::*;
use spin::{Mutex, MutexGuard};

pub struct StdoutLock<'a>(MutexGuard<'a, ()>);

impl<'a> core::fmt::Write for StdoutLock<'a> {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        P::io_write(s);
        Ok(())
    }
}

pub struct Stdout {
    lock: Mutex<()>,
}

impl Stdout {
    const fn new() -> Self {
        Self {
            lock: Mutex::new(()),
        }
    }
    pub fn lock(&self) -> StdoutLock<'_> {
        StdoutLock(self.lock.lock())
    }
}

impl core::fmt::Write for Stdout {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        self.lock().write_str(s)
    }
}

pub fn stdout() -> &'static Stdout {
    static STDOUT: Stdout = Stdout::new();
    &STDOUT
}
