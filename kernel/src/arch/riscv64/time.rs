use crate::prelude::*;
use arch::cpu::{checked_local, local};
use arch::macros::thread_pointer;
use core::ops::Add;
use core::time::Duration;
use riscv::register::time;

#[derive(Debug, Clone, Copy, derive_more::Add)]
pub struct SystemTime(u64);

impl SystemTime {
    pub const ZERO: Self = Self(0);
    pub fn now() -> Self {
        SystemTime(time::read64())
    }
    pub const fn into_raw(self) -> u64 {
        self.0
    }
    pub fn checked_duration_since(self, earlier: Self) -> Option<Duration> {
        if thread_pointer!() == 0 {
            return None;
        }
        let freq = checked_local()?.config().get_frequency()?.frequency;
        Some(Duration::from_micros(
            (self.0 - earlier.0) * 1_000_000 / freq,
        ))
    }
    pub fn duration_since(self, earlier: Self) -> Duration {
        self.checked_duration_since(earlier).unwrap()
    }
}

impl Add<Duration> for SystemTime {
    type Output = SystemTime;

    fn add(self, rhs: Duration) -> Self::Output {
        let freq = local().config().get_frequency().unwrap().frequency;
        SystemTime(self.0 + rhs.as_micros() as u64 * (freq / 1_000_000))
    }
}

impl !Send for SystemTime {}
