use super::errors::BoxError;
use crate::prelude::*;
use core::alloc::Layout;
use core::marker::Unsize;
use core::ops::CoerceUnsized;
use core::ptr::Pointee;
use mem::frames;
use mem::vmm::KMap;

pub struct PhysBox<T: ?Sized>(*mut T);
// it seems that coerce_unsized and ptr_metadata play not well

unsafe impl<T: ?Sized + Send> Send for PhysBox<T> {}
unsafe impl<T: ?Sized + Sync> Sync for PhysBox<T> {}

impl<T: ?Sized + Unsize<U>, U: ?Sized> CoerceUnsized<PhysBox<U>> for PhysBox<T> {}

impl<T> PhysBox<T> {
    pub fn new(x: T) -> Result<Self, BoxError> {
        let layout = Layout::new::<T>().pad_to_align();
        let layout = MapLayout::new(layout.size(), layout.align()).unwrap();
        let paddr = frames::alloc(layout)?;
        unsafe {
            (paddr.to_usize() as *mut T).write_volatile(x);
        }
        Ok(PhysBox(paddr.to_usize() as *mut T))
    }
}

impl<T: ?Sized> PhysBox<T> {
    pub unsafe fn new_zeroed_unsize(metadata: <T as Pointee>::Metadata) -> Result<Self, BoxError> {
        let nullptr = core::ptr::from_raw_parts::<T>(core::ptr::null(), metadata);
        let layout = core::alloc::Layout::for_value_raw(nullptr).pad_to_align();
        let layout = MapLayout::new(layout.size(), layout.align()).unwrap();
        let paddr = frames::alloc(layout)?;
        let data_address = paddr.to_usize() as *mut u8;
        core::slice::from_raw_parts_mut(data_address, layout.size()).fill(0);
        let boxptr = core::ptr::from_raw_parts_mut(data_address as *mut (), metadata) as *mut T;
        Ok(PhysBox(boxptr))
    }
    pub fn paddr(&self) -> PAddr {
        PAddr::new(self.0 as *mut () as usize)
    }
    pub fn into_raw(self) -> PAddr {
        let raw = self.0 as *mut () as usize;
        core::mem::forget(self);
        PAddr::new(raw)
    }
    pub const unsafe fn from_raw(addr: PAddr, metadata: <T as Pointee>::Metadata) -> PhysBox<T> {
        let addr = addr.to_usize() as *mut ();
        PhysBox(core::ptr::from_raw_parts_mut(addr, metadata))
    }
    pub fn get(&self) -> *mut T {
        self.0
    }
}

impl<T: ?Sized> Drop for PhysBox<T> {
    fn drop(&mut self) {
        unsafe {
            core::ptr::drop_in_place(self.get());
            frames::dealloc(self.paddr(), self.layout());
        }
    }
}

impl<T: ?Sized> Map for PhysBox<T> {
    fn layout(&self) -> MapLayout {
        unsafe {
            let layout = Layout::for_value_raw(self.get()).pad_to_align();
            MapLayout::new(layout.size(), layout.align()).unwrap()
        }
    }
}

impl<T: ?Sized> MapIndex for PhysBox<T> {
    unsafe fn index_unchecked(&self, i: usize) -> PAddr {
        self.paddr() + i * self.layout().align()
    }
}

unsafe impl<T: Send + Sync> KMap for PhysBox<T> {}
