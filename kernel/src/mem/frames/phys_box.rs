use super::errors::BoxError;
use crate::prelude::*;
use core::alloc::Layout;
use core::marker::PhantomData;
use mem::frames::FRAMES;
use mem::vmm::KMap;

pub struct PhysBox<T> {
    paddr: PAddr,
    _maker: PhantomData<T>,
}

unsafe impl<T: Send> Send for PhysBox<T> {}
unsafe impl<T: Sync> Sync for PhysBox<T> {}

impl<T> PhysBox<T> {
    pub fn new(x: T) -> Result<PhysBox<T>, BoxError> {
        let layout = Self::layout();
        let paddr = FRAMES.alloc(layout)?;
        unsafe {
            (paddr.to_usize() as *mut T).write_volatile(x);
        }
        Ok(PhysBox {
            paddr,
            _maker: PhantomData,
        })
    }

    pub const fn paddr(&self) -> PAddr {
        self.paddr
    }

    pub const fn into_raw(self) -> PAddr {
        let raw = self.paddr;
        core::mem::forget(self);
        raw
    }

    pub const unsafe fn from_raw(raw: PAddr) -> PhysBox<T> {
        PhysBox {
            paddr: raw,
            _maker: PhantomData,
        }
    }

    pub const fn get(&self) -> *mut T {
        self.paddr.to_usize() as *mut T
    }

    pub fn layout() -> MapLayout {
        let need_pad = Layout::new::<T>().pad_to_align();
        MapLayout::new(need_pad.size(), need_pad.align()).unwrap()
    }
}

impl<T> Drop for PhysBox<T> {
    fn drop(&mut self) {
        unsafe {
            core::ptr::drop_in_place(self.get());
        }
        FRAMES.dealloc(self.paddr, Self::layout());
    }
}

impl<T> Map for PhysBox<T> {
    fn layout(&self) -> MapLayout {
        PhysBox::<T>::layout()
    }
}

impl<T> MapIndex for PhysBox<T> {
    unsafe fn index_unchecked(&self, i: usize) -> PAddr {
        self.paddr + i * Self::layout().align()
    }
}

unsafe impl<T: Send + Sync> KMap for PhysBox<T> {}
