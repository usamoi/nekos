use crate::prelude::*;
use mem::frames;
use mem::frames::FramesAllocError;
use proc::vmm::MapUser;

#[derive(Debug, Clone)]
pub enum MemoryCreateError {
    UndersizeAlign,
    OutOfMemory,
}

fully!(FramesAllocError, MemoryCreateError; OutOfMemory, UndersizeAlign);

pub struct Memory {
    paddr: Box<[PAddr]>,
    layout: MapLayout,
}

impl Memory {
    pub fn create(layout: MapLayout) -> Result<Arc<Memory>, MemoryCreateError> {
        let point = MapLayout::new(layout.align(), layout.align()).unwrap();
        let mut points = Vec::new();
        points.reserve(layout.size() / layout.align());
        for _ in 0..layout.size() / layout.align() {
            match frames::alloc(point) {
                Ok(paddr) => {
                    points.push(paddr);
                }
                Err(e) => {
                    for paddr in points.into_iter() {
                        unsafe {
                            frames::dealloc(paddr, point);
                        }
                    }
                    return Err(e.into());
                }
            }
        }
        Ok(Arc::new(Memory {
            paddr: points.into_boxed_slice(),
            layout,
        }))
    }
}

impl Map for Memory {
    fn layout(&self) -> MapLayout {
        self.layout
    }
}

impl MapRead for Memory {
    unsafe fn read_unchecked(&self, offset: usize, buffer: &mut [u8]) {
        let m = self.layout.align();
        let mut ptr = offset;
        while ptr < offset + buffer.len() {
            let r = usize::min((ptr | (m - 1)) + 1, offset + buffer.len());
            let data = self.paddr[ptr / m].to_const().add(ptr & (m - 1));
            let src = core::slice::from_raw_parts(data, r - ptr);
            buffer[ptr - offset..r - offset].copy_from_slice(src);
            ptr = r;
        }
    }
}

impl MapWrite for Memory {
    unsafe fn write_unchecked(&self, offset: usize, buffer: &[u8]) {
        let m = self.layout.align();
        let mut ptr = offset;
        while ptr < offset + buffer.len() {
            let r = usize::min((ptr | (m - 1)) + 1, offset + buffer.len());
            let data = self.paddr[ptr / m].to_mut().add(ptr & (m - 1));
            let dest = core::slice::from_raw_parts_mut(data, r - ptr);
            dest.copy_from_slice(&buffer[ptr - offset..r - offset]);
            ptr = r;
        }
    }
}

impl MapIndex for Memory {
    unsafe fn index_unchecked(&self, i: usize) -> PAddr {
        self.paddr[i]
    }
}

impl MapUser for Memory {}
