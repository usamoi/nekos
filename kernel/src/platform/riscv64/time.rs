use crate::prelude::*;
use core::time::Duration;
use riscv::register::time;
use rt::time::TimeSource;

pub struct HartTime {
    pub freq: u64,
}

impl TimeSource for HartTime {
    fn now(&self) -> u64 {
        time::read64()
    }

    fn add(&self, value: u64, delta: Duration) -> u64 {
        value + delta.as_micros() as u64 * (self.freq / 1_000_000)
    }

    fn distance(&self, segment: Segment<u64>) -> Duration {
        Duration::from_micros((segment.wrapping_end() - segment.start()) * 1_000_000 / self.freq)
    }

    fn timer(&self, value: u64) {
        super::sbi::timer_set_timer(value).unwrap();
    }
}

impl !Send for HartTime {}
impl !Sync for HartTime {}
