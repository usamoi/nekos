use crate::prelude::*;
use alloc::collections::BTreeMap;
use base::thread::ThreadLocalRef;
use rt::time::Instant;
use spin::Once;

#[derive(Debug, Clone, Copy)]
pub struct Extra {
    pub frequency: u64,
}

pub struct Thread {
    pub stack: Segment<usize>,
    pub extra: Extra,
}

static THREADS: Once<BTreeMap<usize, Thread>> = Once::new();

pub fn maybe_threads() -> Option<&'static BTreeMap<usize, Thread>> {
    THREADS.get()
}

pub fn threads() -> &'static BTreeMap<usize, Thread> {
    maybe_threads().unwrap()
}

pub fn hook_set_threads(threads: BTreeMap<usize, Thread>) {
    THREADS.call_once(|| threads);
}

pub struct Current {}

impl Current {
    pub const fn new() -> Self {
        Self {}
    }
    pub fn id(&self) -> usize {
        P::thread_id()
    }
    pub fn flush_ins(&self) {
        P::thread_flush_ins();
    }
    pub fn flush_tlb(&self) {
        P::thread_flush_tlb();
    }
    pub fn set_timer(&self, time: Instant) {
        P::time_timer(time.value());
    }
}

#[thread_local]
static CURRENT: Current = Current::new();

pub fn current() -> ThreadLocalRef<Current> {
    unsafe { ThreadLocalRef::new(&CURRENT) }
}
