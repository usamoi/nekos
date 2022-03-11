use crate::prelude::*;
use core::alloc::GlobalAlloc;
use core::alloc::Layout;
use core::ptr::{null_mut, NonNull};
use nekos_heap::fallback::Heap as FallbackHeap;
use nekos_heap::slab::Heap as SlabHeap;
use nekos_heap::Mmap;
use spin::Mutex;

#[link_section = ".uninit"]
static mut FALLBACK: [u8; config::FALLBACK_HEAP_SIZE] = [0; config::FALLBACK_HEAP_SIZE];

#[global_allocator]
pub static HEAP: Heap = Heap::new();

pub struct SlabMmap;

impl Mmap for SlabMmap {
    fn map(vaddr: usize) {
        let vaddr = VAddr::new(vaddr);
        let layout = MapLayout::new(4096, 4096).unwrap();
        let paddr = mem::frames::FRAMES.alloc(layout).unwrap();
        mem::vmm::SPACE
            .page_table
            .map(vaddr, paddr, 4096, MapPermission::RW, false, false)
            .unwrap();
    }

    fn unmap(vaddr: usize) {
        let vaddr = VAddr::new(vaddr);
        let layout = MapLayout::new(4096, 4096).unwrap();
        let paddr = mem::vmm::SPACE.page_table.unmap(vaddr, 4096).unwrap();
        mem::frames::FRAMES.dealloc(paddr, layout).unwrap();
    }
}

struct HeapInner {
    slab: Option<SlabHeap<SlabMmap>>,
    fallback: FallbackHeap,
}

pub struct Heap {
    inner: Mutex<HeapInner>,
}

impl Heap {
    pub const fn new() -> Self {
        Self {
            inner: Mutex::new(HeapInner {
                slab: None,
                fallback: FallbackHeap::new(),
            }),
        }
    }
    pub fn init_fallback(&self) {
        let mut inner = self.inner.lock();
        unsafe {
            let start = FALLBACK.as_mut_ptr() as usize;
            let end = start + FALLBACK.len();
            inner.fallback.init(start, end);
        }
    }
    pub fn init_slab(&self) {
        let mut inner = self.inner.lock();
        let segment = mem::vmm::SPACE.heap.segment;
        let start = segment.start().to_usize();
        let end = segment.end().unwrap().to_usize();
        let slab = SlabHeap::<SlabMmap>::new(start, end);
        inner.slab = Some(slab);
    }
}

unsafe impl GlobalAlloc for Heap {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let mut inner = self.inner.lock();
        if let Some(slab) = &mut inner.slab {
            if let Some(addr) = slab.alloc(layout) {
                return addr.as_ptr();
            }
        }
        if let Some(addr) = inner.fallback.alloc(layout) {
            return addr.as_ptr();
        }
        null_mut()
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        let ptr = NonNull::new(ptr).unwrap();
        let mut inner = self.inner.lock();
        if let Some(slab) = &mut inner.slab {
            if slab.test(ptr) {
                slab.dealloc(ptr, layout);
                return;
            }
        }
        if inner.fallback.test(ptr) {
            inner.fallback.dealloc(ptr, layout);
            return;
        }
        panic!("dealloc a unknown address, ptr = {:#p}", ptr);
    }
}
