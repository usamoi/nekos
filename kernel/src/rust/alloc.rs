use crate::prelude::*;
use core::alloc::Allocator;
use core::alloc::GlobalAlloc;
use core::alloc::Layout;
use core::ptr::null_mut;
use core::ptr::NonNull;
use linked_list_allocator::Heap;
use spin::Mutex;

#[lang = "oom"]
fn alloc_error_handler(layout: Layout) -> ! {
    panic!("alloc error, layout = {:?}", layout);
}

static mut BUFFER: [u8; config::FALLBACK_HEAP_SIZE] = [0; config::FALLBACK_HEAP_SIZE];

static FALLBACK: Mutex<Heap> = Mutex::new(Heap::empty());

pub unsafe fn init_global() {
    FALLBACK.lock().init(BUFFER.as_ptr() as usize, BUFFER.len());
}

pub struct GlobalAllocator;

#[global_allocator]
static GLOBAL_ALLOCATOR: GlobalAllocator = GlobalAllocator;

unsafe impl GlobalAlloc for GlobalAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        if layout.size() == 0 {
            layout.align() as *mut u8
        } else if let Ok(ptr) = mem::heap::DefaultAllocator.allocate(layout) {
            ptr.as_ptr().as_mut_ptr()
        } else if let Some(ptr) = FALLBACK.lock().allocate_first_fit(layout).ok() {
            ptr.as_ptr()
        } else {
            null_mut()
        }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        let ptr = NonNull::new(ptr).unwrap();
        if layout.size() == 0 {
            assert_eq!(ptr.as_ptr(), layout.align() as *mut u8);
        } else if BUFFER.as_ptr_range().contains(&(ptr.as_ptr() as *const _)) {
            FALLBACK.lock().deallocate(ptr, layout);
        } else {
            mem::heap::DefaultAllocator.deallocate(ptr, layout);
        }
    }
}
