use crate::prelude::*;
use alloc::collections::BTreeMap;
use core::alloc::{AllocError, Allocator, Layout};
use core::ptr::NonNull;
use mem::frames;
use spin::Mutex;

fn dma_alloc(size: usize, align: usize) -> usize {
    frames::alloc(MapLayout::new(size, align).unwrap())
        .unwrap()
        .to_usize()
}

fn dma_dealloc(addr: usize, size: usize, align: usize) {
    unsafe {
        frames::dealloc(PAddr::new(addr), MapLayout::new(size, align).unwrap());
    }
}

// S <= 4096, T = 4096 / S
struct LinkedList<const S: usize, const T: usize> {
    map: BTreeMap<usize, (u8, [u8; T])>,
}

impl<const S: usize, const T: usize> LinkedList<S, T> {
    const fn new() -> Self {
        Self {
            map: BTreeMap::new(),
        }
    }
    fn alloc(&mut self) -> NonNull<u8> {
        if self.map.is_empty() {
            let xaddr = dma_alloc(4096, 4096);
            let mut arr = [0u8; T];
            for (i, item) in arr.iter_mut().enumerate().take(T - 1) {
                *item = i as u8 + 1;
            }
            arr[T - 1] = T as u8 - 1;
            self.map.insert(xaddr, (0, arr));
        }
        let mut entry = self.map.first_entry().unwrap();
        let key = *entry.key();
        let (head, arr) = entry.get_mut();
        let data_address = NonNull::new((key + *head as usize * S) as *mut _).unwrap();
        if arr[*head as usize] == *head {
            entry.remove();
        } else {
            *head = arr[*head as usize];
        }
        data_address
    }
    fn dealloc(&mut self, addr: usize) {
        let page = addr & !4095;
        let index = (addr & 4095) / S;
        match self.map.get_mut(&page) {
            Some((head, arr)) => {
                arr[index] = *head;
                *head = index as u8;
            }
            None => {
                let mut arr = [0u8; T];
                arr[index] = index as u8;
                self.map.insert(page, (index as u8, arr));
            }
        }
    }
}

static L0: Mutex<LinkedList<16, 256>> = Mutex::new(LinkedList::new());
static L1: Mutex<LinkedList<32, 128>> = Mutex::new(LinkedList::new());
static L2: Mutex<LinkedList<64, 64>> = Mutex::new(LinkedList::new());
static L3: Mutex<LinkedList<128, 32>> = Mutex::new(LinkedList::new());
static L4: Mutex<LinkedList<256, 16>> = Mutex::new(LinkedList::new());
static L5: Mutex<LinkedList<512, 8>> = Mutex::new(LinkedList::new());
static L6: Mutex<LinkedList<1024, 4>> = Mutex::new(LinkedList::new());
static L7: Mutex<LinkedList<2048, 2>> = Mutex::new(LinkedList::new());

pub struct DmaAllocator;

unsafe impl Allocator for DmaAllocator {
    #[allow(clippy::match_overlapping_arm)]
    fn allocate(&self, layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
        let metadata = layout.size();
        let layout = layout.pad_to_align();
        let size = layout.size();
        let align = layout.align();
        let data_address = match size {
            0 => NonNull::new(align as *mut ()).unwrap(),
            1..=16 => L0.lock().alloc().cast(),
            1..=32 => L1.lock().alloc().cast(),
            1..=64 => L2.lock().alloc().cast(),
            1..=128 => L3.lock().alloc().cast(),
            1..=256 => L4.lock().alloc().cast(),
            1..=512 => L5.lock().alloc().cast(),
            1..=1024 => L6.lock().alloc().cast(),
            1..=2048 => L7.lock().alloc().cast(),
            _ => {
                let size = core::cmp::max(4096, size);
                let align = core::cmp::max(4096, align);
                let addr = dma_alloc(size, align);
                NonNull::new(addr as *mut ()).unwrap()
            }
        };
        Ok(NonNull::from_raw_parts(data_address, metadata))
    }

    #[allow(clippy::match_overlapping_arm)]
    unsafe fn deallocate(&self, ptr: NonNull<u8>, layout: Layout) {
        let layout = layout.pad_to_align();
        let size = layout.size();
        let align = layout.align();
        let addr = ptr.as_ptr() as usize;
        match size {
            0 => assert_eq!(addr, align),
            1..=16 => L0.lock().dealloc(addr),
            1..=32 => L1.lock().dealloc(addr),
            1..=64 => L2.lock().dealloc(addr),
            1..=128 => L3.lock().dealloc(addr),
            1..=256 => L4.lock().dealloc(addr),
            1..=512 => L5.lock().dealloc(addr),
            1..=1024 => L6.lock().dealloc(addr),
            1..=2048 => L7.lock().dealloc(addr),
            _ => {
                let size = core::cmp::max(4096, size);
                let align = core::cmp::max(4096, align);
                dma_dealloc(addr, size, align);
            }
        }
    }
}
