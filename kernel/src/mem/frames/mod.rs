mod buddy;

use crate::prelude::*;
use base::cell::SingletonCell;
use buddy::Buddy;
use core::alloc::Layout;
use spin::Mutex;

#[derive(Debug)]
pub enum FramesAllocError {
    UndersizeAlign,
    OutOfMemory,
}

partially!(buddy::BuddyError, FramesAllocError; OutOfBounds => OutOfMemory);

struct Mod {
    buddy: Mutex<Buddy<'static>>,
}

static MOD: SingletonCell<Mod> = SingletonCell::new();

pub unsafe fn init_global() {
    let memory = rt::mem::memory();
    let segment = by_points(memory.ptr, memory.end).unwrap();
    let buffer_size = memory.buffer.wrapping_end() - memory.buffer.start();
    let buffer_slice =
        core::slice::from_raw_parts_mut(memory.buffer.start().to_usize() as *mut i8, buffer_size);
    let buddy_start = segment.start().to_usize().div_ceil(4096);
    let buddy_end = segment.end().map(|x| x.to_usize() >> 12);
    let buddy_segment = Segment::new(buddy_start, buddy_end).unwrap();
    let buddy = Buddy::new(buddy_segment, buffer_slice).unwrap();
    let allocator = Mod {
        buddy: Mutex::new(buddy),
    };
    MOD.initialize(allocator);
}

// todo: non-continuous alloc
pub fn alloc(layout: MapLayout) -> Result<PAddr, FramesAllocError> {
    use FramesAllocError::*;
    if layout.size() == 0 {
        return Ok(PAddr::new(layout.align()));
    }
    if layout.align() < 4096 {
        return Err(UndersizeAlign);
    }
    let mut buddy = MOD.buddy.lock();
    let paddr = buddy.alloc(layout.size() >> 12).out::<FramesAllocError>()?;
    Ok(PAddr::new(paddr << 12))
}

pub unsafe fn dealloc(paddr: PAddr, layout: MapLayout) {
    if layout.size() == 0 {
        assert_eq!(paddr, PAddr::new(layout.align()));
        return;
    }
    assert!(layout.align() >= 4096);
    assert!(layout.check(paddr.to_usize()));
    let mut buddy = MOD.buddy.lock();
    let addr = paddr.to_usize() >> 12;
    let size = layout.size() >> 12;
    buddy.dealloc(addr, size).unwrap();
}

pub struct FramesBox<T>(*mut T);

unsafe impl<T: Send> Send for FramesBox<T> {}
unsafe impl<T: Sync> Sync for FramesBox<T> {}

impl<T> FramesBox<T> {
    pub fn new(x: T) -> Result<Self, FramesAllocError> {
        let layout = Layout::new::<T>().pad_to_align();
        let layout = MapLayout::new(layout.size(), layout.align()).unwrap();
        let paddr = alloc(layout)?;
        unsafe {
            (paddr.to_usize() as *mut T).write_volatile(x);
        }
        Ok(FramesBox(paddr.to_usize() as *mut T))
    }
}

impl<T> FramesBox<T> {
    pub fn paddr(&self) -> PAddr {
        PAddr::new(self.0 as usize)
    }
    pub fn into_raw(self) -> PAddr {
        let raw = self.0 as usize;
        core::mem::forget(self);
        PAddr::new(raw)
    }
    pub const unsafe fn from_raw(addr: PAddr) -> FramesBox<T> {
        Self(addr.to_usize() as *mut T)
    }
    pub fn get(&self) -> *mut T {
        self.0
    }
    pub fn layout(&self) -> MapLayout {
        let layout = Layout::new::<T>().pad_to_align();
        MapLayout::new(layout.size(), layout.align()).unwrap()
    }
}

impl<T> Drop for FramesBox<T> {
    fn drop(&mut self) {
        unsafe {
            core::ptr::drop_in_place(self.get());
            dealloc(self.paddr(), self.layout());
        }
    }
}
