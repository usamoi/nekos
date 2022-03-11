use crate::prelude::*;
use alloc::collections::BTreeMap;
use proc::thread::Thread;
use spin::Mutex;

pub struct ThreadSet {
    pub threads: Mutex<BTreeMap<usize, Arc<Thread>>>,
}

impl ThreadSet {
    pub fn new() -> Self {
        Self {
            threads: Mutex::new(BTreeMap::new()),
        }
    }
    pub fn insert(&self, x: Arc<Thread>) {
        self.threads.lock().insert(Arc::as_ptr(&x) as usize, x);
    }
    pub fn broadcast(&self, signal: Signal) {
        let inner = self.threads.lock();
        for (_, thread) in inner.iter() {
            thread.signal_set.send(signal.clone());
        }
    }
    pub fn on_thread_killed(&self, thread: &Arc<Thread>) {
        let mut inner = self.threads.lock();
        let value = inner.remove(&(Arc::as_ptr(thread) as usize)).unwrap();
        assert_eq!(Arc::as_ptr(&value), Arc::as_ptr(thread));
    }
}
