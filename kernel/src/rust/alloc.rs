use crate::prelude::*;
use core::alloc::AllocError;
use core::alloc::Allocator;
use core::alloc::GlobalAlloc;
use core::alloc::Layout;
use core::mem::MaybeUninit;
use core::ptr::null_mut;
use core::ptr::NonNull;
use linked_list_allocator::Heap;
use mem::heap::DefaultAllocator;
use spin::Mutex;

#[lang = "oom"]
fn alloc_error_handler(layout: Layout) -> ! {
    panic!("alloc error, layout = {:?}", layout);
}

core::arch::global_asm!(".section    .brk.heap,\"wa\",@nobits");

#[link_section = ".brk.heap"]
static mut HEAP: [MaybeUninit<u8>; config::HEAP_SIZE] = [MaybeUninit::uninit(); config::HEAP_SIZE];

static FALLBACK: Mutex<Heap> = Mutex::new(Heap::empty());

pub unsafe fn init_global() {
    FALLBACK.lock().init(HEAP.as_ptr() as usize, HEAP.len());
}

pub struct FallbackAllocator;

unsafe impl Allocator for FallbackAllocator {
    fn allocate(&self, layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
        match FALLBACK.lock().allocate_first_fit(layout) {
            Ok(ptr) => Ok(NonNull::from_raw_parts(ptr.cast(), layout.size())),
            Err(()) => Err(AllocError),
        }
    }

    unsafe fn deallocate(&self, ptr: NonNull<u8>, layout: Layout) {
        FALLBACK.lock().deallocate(ptr, layout)
    }
}

#[global_allocator]
static GLOBAL_ALLOCATOR: GlobalAllocator = GlobalAllocator;

pub struct GlobalAllocator;

unsafe impl GlobalAlloc for GlobalAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        if layout.size() == 0 {
            layout.align() as *mut u8
        } else if let Ok(ptr) = DefaultAllocator.allocate(layout) {
            ptr.as_ptr().cast()
        } else if let Ok(ptr) = FallbackAllocator.allocate(layout) {
            ptr.as_ptr().cast()
        } else {
            null_mut()
        }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        let ptr = NonNull::new(ptr).unwrap();
        if layout.size() == 0 {
            assert_eq!(ptr.as_ptr(), layout.align() as *mut u8);
        } else if HEAP.as_ptr_range().contains(&(ptr.as_ptr() as *const _)) {
            FallbackAllocator.deallocate(ptr, layout);
        } else {
            DefaultAllocator.deallocate(ptr, layout);
        }
    }
}
