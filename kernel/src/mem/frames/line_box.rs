use super::errors::BoxError;
use crate::prelude::*;
use mem::frames::FRAMES;
use mem::vmm::KMap;

pub struct LineBox {
    paddr: PAddr,
    layout: MapLayout,
}

impl LineBox {
    pub fn new(layout: MapLayout) -> Result<LineBox, BoxError> {
        Ok(LineBox {
            paddr: FRAMES.alloc(layout)?,
            layout,
        })
    }

    pub const fn paddr(&self) -> PAddr {
        self.paddr
    }
}

impl Map for LineBox {
    fn layout(&self) -> MapLayout {
        self.layout
    }
}

impl RandomRead for LineBox {
    unsafe fn read_unchecked(&self, offset: usize, buffer: &mut [u8]) {
        let slice = core::slice::from_raw_parts(self.paddr.to_const(), self.layout.size());
        buffer.copy_from_slice(&slice[offset..offset + buffer.len()]);
    }
}

impl RandomWrite for LineBox {
    unsafe fn write_unchecked(&self, offset: usize, buffer: &[u8]) {
        let slice = core::slice::from_raw_parts_mut(self.paddr.to_mut(), self.layout.size());
        slice[offset..offset + buffer.len()].copy_from_slice(buffer);
    }
}

impl MapIndex for LineBox {
    unsafe fn index_unchecked(&self, i: usize) -> PAddr {
        self.paddr + i * self.layout().align()
    }
}

unsafe impl KMap for LineBox {}
