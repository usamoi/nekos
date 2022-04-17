use crate::base::thread::ThreadLocalRef;
use crate::prelude::*;
use base::cell::SingletonCell;
use core::ops::{Add, Sub};
use core::time::Duration;

pub trait TimeSource {
    fn now(&self) -> u64;
    fn add(&self, _: u64, delta: Duration) -> u64;
    fn distance(&self, _: Segment<u64>) -> Duration;
    fn timer(&self, _: u64);
}

#[thread_local]
static LOCAL: SingletonCell<Box<dyn TimeSource>> = SingletonCell::new();

pub fn init_local(source: Box<dyn TimeSource>) {
    LOCAL.initialize(source);
}

pub fn maybe_local() -> Option<ThreadLocalRef<dyn TimeSource>> {
    unsafe { Some(ThreadLocalRef::new(LOCAL.maybe()?.as_ref())) }
}

pub fn local() -> ThreadLocalRef<dyn TimeSource> {
    maybe_local().unwrap()
}

#[derive(Debug, Clone, Copy)]
pub struct Instant(u64);

impl Instant {
    pub const ZERO: Self = Self(0);
    pub fn maybe_now() -> Option<Self> {
        Some(Instant(maybe_local()?.now()))
    }
    pub fn now() -> Self {
        Self::maybe_now().unwrap()
    }
    pub fn value(self) -> u64 {
        self.0
    }
}

impl Add<Duration> for Instant {
    type Output = Instant;

    fn add(self, rhs: Duration) -> Self::Output {
        Self(local().add(self.0, rhs))
    }
}

impl Sub<Instant> for Instant {
    type Output = Duration;

    fn sub(self, rhs: Instant) -> Self::Output {
        local().distance(by_points(rhs.0, self.0).unwrap())
    }
}

impl !Send for Instant {}
impl !Sync for Instant {}
