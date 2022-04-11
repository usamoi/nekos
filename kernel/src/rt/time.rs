use crate::prelude::*;
use core::ops::Add;
use core::time::Duration;
use riscv::register::time;

#[derive(Debug, Clone, Copy)]
pub struct Instant(u64);

impl Instant {
    pub const ZERO: Self = Self(0);
    pub fn now() -> Self {
        Instant(time::read64())
    }
    pub const fn value(self) -> u64 {
        self.0
    }
    pub fn maybe_duration_since(self, earlier: Self) -> Option<Duration> {
        P::time_sub_maybe(earlier.0, self.0)
    }
    pub fn duration_since(self, earlier: Self) -> Duration {
        self.maybe_duration_since(earlier).unwrap()
    }
}

impl Add<Duration> for Instant {
    type Output = Instant;

    fn add(self, rhs: Duration) -> Self::Output {
        Self(P::time_add_maybe(self.0, rhs).unwrap())
    }
}

impl !Send for Instant {}
impl !Sync for Instant {}
