use crate::prelude::*;
use alloc::collections::BTreeMap;
use base::cell::SingletonCell;
use base::thread::ThreadLocalRef;

pub struct Thread {}

static THREADS: SingletonCell<BTreeMap<usize, Thread>> = SingletonCell::new();

pub fn maybe_threads() -> Option<&'static BTreeMap<usize, Thread>> {
    THREADS.maybe()
}

pub fn threads() -> &'static BTreeMap<usize, Thread> {
    maybe_threads().unwrap()
}

pub fn init_global(threads: BTreeMap<usize, Thread>) {
    THREADS.initialize(threads);
}

pub struct Current {}

impl Current {
    pub const fn new() -> Self {
        Self {}
    }
    pub fn id(&self) -> usize {
        P::id()
    }
}

#[thread_local]
static CURRENT: Current = Current::new();

pub fn current() -> ThreadLocalRef<Current> {
    unsafe { ThreadLocalRef::new(&CURRENT) }
}
