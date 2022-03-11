use crate::prelude::*;

pub unsafe trait KMap: Send + Sync + Map + MapIndex {}

pub struct KMapUnsafe {
    paddr: PAddr,
    layout: MapLayout,
}

impl KMapUnsafe {
    pub unsafe fn new(paddr: PAddr, layout: MapLayout) -> Option<Self> {
        if !layout.check(paddr.to_usize()) {
            return None;
        }
        Some(Self { paddr, layout })
    }
}

impl Map for KMapUnsafe {
    fn layout(&self) -> MapLayout {
        self.layout
    }
}

impl MapIndex for KMapUnsafe {
    unsafe fn index_unchecked(&self, i: usize) -> PAddr {
        assert!(i < self.layout.size() / self.layout.align());
        self.paddr + self.layout.align() * i
    }
}

unsafe impl KMap for KMapUnsafe {}
