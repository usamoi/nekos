use crate::prelude::*;
use mem::dma::DmaBox;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum RawDma {
    Ref(RawDmaRef),
    Mut(RawDmaMut),
}

impl RawDma {
    pub fn paddr(self) -> PAddr {
        match self {
            RawDma::Ref(RawDmaRef(paddr, _)) => paddr,
            RawDma::Mut(RawDmaMut(paddr, _)) => paddr,
        }
    }
    pub fn size(self) -> usize {
        match self {
            RawDma::Ref(RawDmaRef(_, size)) => size,
            RawDma::Mut(RawDmaMut(_, size)) => size,
        }
    }
    pub fn is_writable(self) -> bool {
        match self {
            RawDma::Ref(_) => false,
            RawDma::Mut(_) => true,
        }
    }
}

impl From<RawDmaRef> for RawDma {
    fn from(x: RawDmaRef) -> Self {
        RawDma::Ref(x)
    }
}

impl From<RawDmaMut> for RawDma {
    fn from(x: RawDmaMut) -> Self {
        RawDma::Mut(x)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct RawDmaRef(pub PAddr, pub usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct RawDmaMut(pub PAddr, pub usize);

pub trait AsRawDma {
    fn as_raw_dma_ref(&self) -> RawDmaRef;
    fn as_raw_dma_mut(&self) -> RawDmaMut;
}

impl<T: ?Sized> AsRawDma for DmaBox<T> {
    fn as_raw_dma_ref(&self) -> RawDmaRef {
        let paddr = PAddr::new(self.as_ref() as *const T as *const () as usize);
        let size = core::mem::size_of_val(self.as_ref());
        RawDmaRef(paddr, size)
    }
    fn as_raw_dma_mut(&self) -> RawDmaMut {
        let paddr = PAddr::new(self.as_ref() as *const T as *const () as usize);
        let size = core::mem::size_of_val(self.as_ref());
        RawDmaMut(paddr, size)
    }
}
