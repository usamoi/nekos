use crate::prelude::*;
use alloc::collections::BTreeMap;
use crossbeam::atomic::AtomicCell;
use spin::Mutex;

pub struct HandleSet {
    count: AtomicCell<usize>,
    inner: Mutex<BTreeMap<HandleID, Handle>>,
}

impl HandleSet {
    pub fn new() -> HandleSet {
        HandleSet {
            count: AtomicCell::new(config::PROCESS_RESERVE_HANDLES),
            inner: Mutex::new(BTreeMap::new()),
        }
    }
    #[must_use]
    pub fn extend(&self, id: HandleID, handle: Handle) -> Option<Handle> {
        self.inner.lock().insert(id, handle)
    }
    pub fn push(&self, handle: Handle) -> HandleID {
        let id = self.count.fetch_add(1);
        assert_ne!(id, 0);
        self.inner.lock().insert(id, handle);
        id
    }
    pub fn lookup(&self, id: HandleID) -> Option<Handle> {
        self.inner.lock().get(&id).cloned()
    }
    pub fn remove(&self, id: HandleID) -> Option<Handle> {
        self.inner.lock().remove(&id)
    }
}
