use core::alloc::Layout;
use core::ptr::NonNull;
use linked_list_allocator::Heap as Linkedlist;

pub struct Heap {
    start: usize,
    end: usize,
    linkedlist: Linkedlist,
}

impl Heap {
    pub const fn new() -> Self {
        Self {
            start: 0,
            end: 0,
            linkedlist: Linkedlist::empty(),
        }
    }
    pub unsafe fn init(&mut self, start: usize, end: usize) {
        self.start = start;
        self.end = end;
        self.linkedlist.init(start, end - start);
    }
    pub fn test(&self, addr: NonNull<u8>) -> bool {
        let addr = addr.as_ptr() as usize;
        self.start <= addr && addr < self.end
    }
    pub fn alloc(&mut self, layout: Layout) -> Option<NonNull<u8>> {
        self.linkedlist.allocate_first_fit(layout).ok()
    }
    pub unsafe fn dealloc(&mut self, addr: NonNull<u8>, layout: Layout) {
        self.linkedlist.deallocate(addr, layout);
    }
}
