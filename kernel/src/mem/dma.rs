use crate::prelude::*;
use buddy_system_allocator::{Heap, LockedHeapWithRescue};
use core::alloc::AllocError;
use core::alloc::Allocator;
use core::ptr::NonNull;

// todo: free dma regions

static DMA: Dma = Dma {
    allocator: Singleton::new(),
};

struct Dma {
    allocator: Singleton<LockedHeapWithRescue<32>>,
}

fn rescue(heap: &mut Heap<32>, layout: &core::alloc::Layout) {
    let layout = layout.pad_to_align();
    let align = core::cmp::max(4096, layout.align());
    let size = layout.size().next_multiple_of(align);
    let layout = MapLayout::new(size, align).unwrap();
    let paddr = mem::frames::alloc(layout).unwrap();
    unsafe {
        heap.add_to_heap(paddr.to_usize(), layout.size());
    }
}

pub struct DmaAllocator;

unsafe impl Allocator for DmaAllocator {
    fn allocate(
        &self,
        layout: core::alloc::Layout,
    ) -> Result<core::ptr::NonNull<[u8]>, AllocError> {
        let p = DMA.allocator.lock().alloc(layout).map_err(|_| AllocError)?;
        let (raw, ()) = p.to_raw_parts();
        Ok(NonNull::from_raw_parts(raw, layout.size()))
    }

    unsafe fn deallocate(&self, ptr: core::ptr::NonNull<u8>, layout: core::alloc::Layout) {
        DMA.allocator.lock().dealloc(ptr, layout)
    }
}

pub fn init_global() {
    DMA.allocator.init(LockedHeapWithRescue::new(rescue));
}

pub trait AsRawDma {
    fn as_raw_dma(&self) -> (PAddr, usize);
}

impl<T: ?Sized> AsRawDma for Box<T, DmaAllocator> {
    fn as_raw_dma(&self) -> (PAddr, usize) {
        (
            PAddr::new(self.as_ref() as *const T as *const () as usize),
            core::mem::size_of_val(self),
        )
    }
}
