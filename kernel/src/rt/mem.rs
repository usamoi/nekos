use crate::prelude::*;
use base::cell::SingletonCell;

pub struct MemoryBuilder {
    pub start: PAddr,
    pub ptr: PAddr,
    pub end: PAddr,
    pub buffer: Option<Segment<PAddr>>,
}

impl MemoryBuilder {
    pub fn new(segment: Segment<PAddr>) -> MemoryBuilder {
        MemoryBuilder {
            start: segment.start(),
            ptr: segment.start(),
            end: segment.end().unwrap(),
            buffer: None,
        }
    }
    pub fn brk(&mut self, addr: PAddr) {
        self.ptr = core::cmp::max(self.ptr, addr);
        assert!(self.ptr <= self.end);
    }
    pub fn sbrk(&mut self, size: usize) -> Segment<PAddr> {
        let ptr = self.ptr;
        self.ptr = ptr + size;
        assert!(self.ptr <= self.end);
        by_size(ptr, size).unwrap()
    }
    pub fn alloc(&mut self, layout: MapLayout) -> PAddr {
        self.ptr = self.ptr.to_usize().next_multiple_of(layout.align()).into();
        let ans = self.ptr;
        self.ptr = self.ptr + layout.size();
        assert!(self.ptr <= self.end);
        ans
    }
    pub fn alloc_buffer(&mut self) {
        let buffer = self.sbrk((self.end - self.start) / 4096 * 2);
        self.buffer = Some(buffer);
    }
    pub fn finish(mut self) -> Option<Memory> {
        self.ptr = self.ptr.to_usize().next_multiple_of(4096).into();
        assert!(self.ptr <= self.end);
        Some(Memory {
            start: self.start,
            ptr: self.ptr,
            end: self.end,
            buffer: self.buffer?,
        })
    }
}

pub struct Memory {
    pub start: PAddr,
    pub ptr: PAddr,
    pub end: PAddr,
    pub buffer: Segment<PAddr>,
}

static MEMORY: SingletonCell<Memory> = SingletonCell::new();

pub fn maybe_memory() -> Option<&'static Memory> {
    MEMORY.maybe()
}

pub fn memory() -> &'static Memory {
    maybe_memory().unwrap()
}

pub fn init_global(region: MemoryBuilder) {
    MEMORY.initialize(region.finish().unwrap());
}
