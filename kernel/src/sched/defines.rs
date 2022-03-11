use crate::prelude::*;
use core::task::{Poll, Waker};
use core::time::Duration;
use crossbeam::atomic::AtomicCell;
use sched::scheduler::make_waker;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Priority(u32);

impl Priority {
    pub const MIN: Priority = Priority(0);
    pub const DEFAULT: Priority = Priority(1000);
    pub const MAX: Priority = Priority(1000000);
    pub fn new(x: u32) -> Option<Priority> {
        if Self::MIN.0 <= x && x <= Self::MAX.0 {
            Some(Priority(x))
        } else {
            None
        }
    }
    pub fn into(self) -> u32 {
        self.0
    }
}

pub trait Pollable: Send + Sync {
    fn poll(&self, waker: Waker, time: Duration) -> Poll<()>;
}

pub struct Task {
    pollable: Arc<dyn Pollable>,
    vruntime: u64,
    priority: Priority,
    block: AtomicCell<usize>,
}

impl Task {
    pub fn new(pollable: Arc<dyn Pollable>, priority: Priority) -> Arc<Task> {
        Arc::new(Task {
            pollable,
            vruntime: 0,
            priority,
            block: AtomicCell::new(0),
        })
    }
    pub unsafe fn poll(&self, waker: impl Fn(), time: Duration) -> Poll<()> {
        self.pollable.poll(make_waker(waker), time)
    }
    pub unsafe fn block(&self) {
        self.block.fetch_add(1);
    }
    pub unsafe fn unblock(&self) {
        self.block.fetch_sub(1);
    }
    pub fn is_blocked(&self) -> bool {
        self.block.load() != 0
    }
}
