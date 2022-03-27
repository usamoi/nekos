use crate::prelude::*;
use arch::abi::thread_pointer;
use arch::cpu::{checked_local, local};
use core::ops::Add;
use core::time::Duration;
use riscv::register::time;

#[derive(Debug, Clone, Copy, derive_more::Add)]
pub struct MachineInstant(u64);

impl MachineInstant {
    pub const ZERO: Self = Self(0);
    pub fn now() -> Self {
        MachineInstant(time::read64())
    }
    pub const fn value(self) -> u64 {
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

impl Add<Duration> for MachineInstant {
    type Output = MachineInstant;

    fn add(self, rhs: Duration) -> Self::Output {
        let freq = local().config().get_frequency().unwrap().frequency;
        MachineInstant(self.0 + rhs.as_micros() as u64 * (freq / 1_000_000))
    }
}

impl !Send for MachineInstant {}
