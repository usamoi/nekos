mod errors;

pub use self::errors::*;

use crate::prelude::*;
use mem::frames::PointsBox;
use proc::vmm::MapProc;

pub struct Memory {
    points: PointsBox,
}

impl Memory {
    pub fn create(layout: MapLayout) -> Result<Arc<Memory>, MemoryCreateError> {
        let points = PointsBox::new(layout)?;
        Ok(Arc::new(Memory { points }))
    }
}

impl Map for Memory {
    fn layout(&self) -> MapLayout {
        self.points.layout()
    }
}

impl RandomRead for Memory {
    unsafe fn read_unchecked(&self, offset: usize, buffer: &mut [u8]) {
        self.points.read_unchecked(offset, buffer)
    }
}

impl RandomWrite for Memory {
    unsafe fn write_unchecked(&self, offset: usize, buffer: &[u8]) {
        self.points.write_unchecked(offset, buffer)
    }
}

impl MapIndex for Memory {
    unsafe fn index_unchecked(&self, i: usize) -> PAddr {
        self.points.index_unchecked(i)
    }
}

impl MapProc for Memory {}
