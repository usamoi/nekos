use super::errors::BoxError;
use crate::prelude::*;
use mem::frames::FRAMES;
use mem::vmm::KMap;

pub struct PointsBox {
    paddr: Box<[PAddr]>,
    layout: MapLayout,
}

impl PointsBox {
    pub fn new(layout: MapLayout) -> Result<PointsBox, BoxError> {
        let point = MapLayout::new(layout.align(), layout.align()).unwrap();
        let mut points = Vec::new();
        points.reserve(layout.size() / layout.align());
        for _ in 0..layout.size() / layout.align() {
            match FRAMES.alloc(point) {
                Ok(paddr) => {
                    points.push(paddr);
                }
                Err(e) => {
                    for paddr in points.into_iter() {
                        FRAMES.dealloc(paddr, point).unwrap();
                    }
                    return Err(e.into());
                }
            }
        }
        Ok(PointsBox {
            paddr: points.into_boxed_slice(),
            layout,
        })
    }
}

impl Map for PointsBox {
    fn layout(&self) -> MapLayout {
        self.layout
    }
}

impl RandomRead for PointsBox {
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

impl RandomWrite for PointsBox {
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

impl MapIndex for PointsBox {
    unsafe fn index_unchecked(&self, i: usize) -> PAddr {
        self.paddr[i]
    }
}

unsafe impl KMap for PointsBox {}
