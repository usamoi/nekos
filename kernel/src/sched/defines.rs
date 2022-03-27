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
    pub fn value(self) -> u128 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Priority(u32);

impl Priority {
    pub const MIN: Self = Self(1);
    pub const MAX: Self = Self(1000000);
    pub const DEFAULT: Self = Self(1000);
    pub fn new(x: u32) -> Option<Self> {
        if Self::MIN.0 <= x && x <= Self::MAX.0 {
            Some(Self(x))
        } else {
            None
        }
    }
    pub fn value(self) -> u32 {
        self.0
    }
}

impl Default for Priority {
    fn default() -> Self {
        Self::DEFAULT
    }
}

pub trait PreemptiveFuture: Send + Sync {
    fn poll(&self, cx: &mut Context, duration: Duration) -> Poll<()>;
}

pub struct Task {
    future: Arc<dyn PreemptiveFuture>,
    vruntime: AtomicCell<Vruntime>,
    priority: AtomicCell<Priority>,
}

impl Task {
    pub fn new(
        future: Arc<dyn PreemptiveFuture>,
        vruntime: Vruntime,
        priority: Priority,
    ) -> Arc<Task> {
        Arc::new(Task {
            future,
            vruntime: AtomicCell::new(vruntime),
            priority: AtomicCell::new(priority),
        })
    }
    pub fn poll(&self, cx: &mut Context, duration: Duration) {
        let step = 1000000000u128 / self.priority().value() as u128;
        let _ = self.future.poll(cx, duration);
        self.set_vruntime(Vruntime::new(self.vruntime().value() + step));
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
    pub fn resched(self: Arc<Task>) {
        let step = 1000000000u128 / self.priority().value() as u128;
        let mut queue = crate::sched::scheduler::SCHEDULER.queue.lock();
        if let Some(queue_vruntime) = queue.vruntime() {
            let limit_num = queue_vruntime.value() + step;
            let limit = Vruntime::new(limit_num);
            self.set_vruntime(core::cmp::max(self.vruntime(), limit));
        }
        queue.insert(self);
    }
}
