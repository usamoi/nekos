use crate::prelude::*;
use core::task::{Context, Poll};
use core::time::Duration;
use crossbeam::atomic::AtomicCell;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Vruntime(u128);

impl Vruntime {
    pub fn new(x: u128) -> Self {
        Self(x)
    }
    pub fn index(self) -> u128 {
        self.0
    }
    // todo: what if duration.as_micros() / priority.index() as u128 == 0?
    pub fn add(self, duration: Duration, priority: Priority) -> Self {
        Self(self.0 + duration.as_micros() / priority.index() as u128)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Priority(u32);

impl Priority {
    pub const MIN: Self = Self(1);
    pub const DEFAULT: Self = Self(1000);
    pub const MAX: Self = Self(1000000);
    pub fn new(x: u32) -> Option<Self> {
        if Self::MIN.0 <= x && x <= Self::MAX.0 {
            Some(Self(x))
        } else {
            None
        }
    }
    pub fn index(self) -> u32 {
        self.0
    }
}

impl Default for Priority {
    fn default() -> Self {
        Self::DEFAULT
    }
}

pub trait TaskFuture: Send + Sync {
    fn poll(&self, cx: &mut Context, duration: Duration) -> Poll<()>;
}

pub struct Task {
    pub(in crate::sched) future: Arc<dyn TaskFuture>,
    vruntime: AtomicCell<Vruntime>,
    priority: AtomicCell<Priority>,
}

impl Task {
    pub fn new(future: Arc<dyn TaskFuture>, vruntime: Vruntime, priority: Priority) -> Arc<Task> {
        Arc::new(Task {
            future,
            vruntime: AtomicCell::new(vruntime),
            priority: AtomicCell::new(priority),
        })
    }
    pub fn vruntime(&self) -> Vruntime {
        self.vruntime.load()
    }
    pub fn set_vruntime(&self, vruntime: Vruntime) {
        self.vruntime.store(vruntime)
    }
    pub fn priority(&self) -> Priority {
        self.priority.load()
    }
    pub fn set_priority(&self, priority: Priority) {
        self.priority.store(priority);
    }
}
