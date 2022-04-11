use crate::prelude::*;
use spin::Once;

pub struct RegionBuilder {
    pub start: PAddr,
    pub ptr: PAddr,
    pub end: PAddr,
    pub use_buffer: Option<Segment<PAddr>>,
}

impl RegionBuilder {
    pub fn new(segment: Segment<PAddr>) -> RegionBuilder {
        RegionBuilder {
            start: segment.start(),
            ptr: segment.start(),
            end: segment.end().unwrap(),
            use_buffer: None,
        }
    }
    pub fn alloc_addr(&mut self, addr: PAddr) {
        self.ptr = core::cmp::max(self.ptr, addr);
        if self.ptr > self.end {
            panic!("region memory overflow");
        }
    }
    pub fn alloc_size(&mut self, size: usize) -> Segment<PAddr> {
        let ptr = self.ptr;
        self.ptr = ptr + size;
        if self.ptr > self.end {
            panic!("region memory overflow");
        }
        by_size(ptr, size).unwrap()
    }
    pub fn set_buffer(&mut self, buffer: Segment<PAddr>) {
        self.use_buffer = Some(buffer);
    }
    pub fn finish(self) -> Option<Region> {
        Some(Region {
            start: self.start,
            ptr: self.ptr,
            end: self.end,
            use_buffer: self.use_buffer?,
        })
    }
}

pub struct Region {
    pub start: PAddr,
    pub ptr: PAddr,
    pub end: PAddr,
    pub use_buffer: Segment<PAddr>,
}

static MEMORY: Once<Region> = Once::new();

pub fn maybe_memory() -> Option<&'static Region> {
    MEMORY.get()
}

pub fn memory() -> &'static Region {
    maybe_memory().unwrap()
}

pub fn hook_set_memory_region(region: Region) {
    MEMORY.call_once(|| region);
}
