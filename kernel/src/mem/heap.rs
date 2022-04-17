use crate::prelude::*;
use core::alloc::AllocError;
use core::alloc::{Allocator, Layout};
use core::ptr::NonNull;
use crossbeam::atomic::AtomicCell;
use mem::vmm::VMM;
use rt::paging::Paging;
use spin::Mutex;

fn def_map(ptr: usize) {
    let layout = MapLayout::new(4096, 4096).unwrap();
    VMM.page_table
        .map(
            VAddr::new(ptr),
            mem::frames::alloc(layout).unwrap(),
            4096,
            Permission::RW,
            false,
            false,
        )
        .unwrap();
    unsafe {
        core::arch::riscv64::sfence_vma(ptr, 0);
    }
}

fn def_unmap(ptr: usize) {
    let layout = MapLayout::new(4096, 4096).unwrap();
    let paddr = VMM.page_table.unmap(VAddr::new(ptr), 4096).unwrap();
    unsafe {
        mem::frames::dealloc(paddr, layout);
        core::arch::riscv64::sfence_vma(ptr, 0);
    }
}

// 32 <= S < 4096, T = S * 65536 / 4096
struct LinkedList<const S: usize, const T: usize> {
    addr: usize,
    count: [u8; T],
    next: [u16; 65536],
    head: Option<u16>,
}

impl<const S: usize, const T: usize> LinkedList<S, T> {
    const fn new() -> Self {
        Self {
            addr: 0,
            count: [0u8; T],
            next: [0u16; 65536],
            head: Some(0),
        }
    }
    fn init(&mut self, addr: &mut usize) {
        self.addr = *addr;
        *addr += S * 65536;
        for i in 0..65535 {
            self.next[i as usize] = i + 1;
        }
        self.next[65535] = 65535;
    }
    fn test(&self, ptr: NonNull<u8>) -> bool {
        let addr = ptr.as_ptr() as usize;
        self.addr <= addr && addr < self.addr + S * 65536
    }
    fn alloc(&mut self) -> Option<NonNull<u8>> {
        let x = self.head.take()?;
        if self.next[x as usize] != x {
            self.head = Some(self.next[x as usize]);
        }
        let page = (x as usize * S) / 4096;
        if self.count[page] == 0 {
            def_map(self.addr + page * 4096);
        }
        self.count[page] += 1;
        Some(NonNull::new((self.addr + (x as usize) * S) as *mut u8).unwrap())
    }
    fn dealloc(&mut self, ptr: NonNull<u8>) {
        assert!(self.test(ptr));
        let x = ((ptr.as_ptr() as usize - self.addr) / S) as u16;
        self.next[x as usize] = self.head.take().unwrap_or(x);
        self.head = Some(x);
        let page = (x as usize * S) / 4096;
        self.count[page] -= 1;
        if self.count[page] == 0 {
            def_unmap(self.addr + page * 4096);
        }
    }
}

// S = 4096k
struct Bitmap<const S: usize> {
    addr: usize,
    bits: u64,
}

impl<const S: usize> Bitmap<S> {
    const fn new() -> Self {
        Self {
            addr: 0,
            bits: u64::MAX,
        }
    }
    fn init(&mut self, addr: &mut usize) {
        self.addr = *addr;
        *addr += S * 64;
    }
    fn test(&self, ptr: NonNull<u8>) -> bool {
        let addr = ptr.as_ptr() as usize;
        self.addr <= addr && addr < self.addr + S * 64
    }
    fn alloc(&mut self) -> Option<NonNull<u8>> {
        let x = self.bits.checked_log2()?;
        self.bits ^= 1 << x;
        for i in 0..(S / 4096) {
            def_map(self.addr + (x as usize) * S + i * 4096);
        }
        Some(NonNull::new((self.addr + x as usize * S) as *mut u8).unwrap())
    }
    fn dealloc(&mut self, ptr: NonNull<u8>) {
        assert!(self.test(ptr));
        let x = (ptr.as_ptr() as usize - self.addr) / S;
        self.bits ^= 1 << x;
        for i in 0..(S / 4096) {
            def_unmap(self.addr + (x as usize) * S + i * 4096);
        }
    }
}

static L1: Mutex<LinkedList<32, { 32 * 65536 / 4096 }>> = Mutex::new(LinkedList::new());
static L2: Mutex<LinkedList<64, { 64 * 65536 / 4096 }>> = Mutex::new(LinkedList::new());
static L3: Mutex<LinkedList<128, { 128 * 65536 / 4096 }>> = Mutex::new(LinkedList::new());
static L4: Mutex<LinkedList<256, { 256 * 65536 / 4096 }>> = Mutex::new(LinkedList::new());
static L5: Mutex<LinkedList<512, { 512 * 65536 / 4096 }>> = Mutex::new(LinkedList::new());
static L6: Mutex<LinkedList<1024, { 1024 * 65536 / 4096 }>> = Mutex::new(LinkedList::new());
static L7: Mutex<LinkedList<2048, { 2048 * 65536 / 4096 }>> = Mutex::new(LinkedList::new());
static B0: Mutex<Bitmap<{ 4 << 10 }>> = Mutex::new(Bitmap::new());
static B1: Mutex<Bitmap<{ 8 << 10 }>> = Mutex::new(Bitmap::new());
static B2: Mutex<Bitmap<{ 16 << 10 }>> = Mutex::new(Bitmap::new());
static B3: Mutex<Bitmap<{ 32 << 10 }>> = Mutex::new(Bitmap::new());
static B4: Mutex<Bitmap<{ 64 << 10 }>> = Mutex::new(Bitmap::new());
static B5: Mutex<Bitmap<{ 128 << 10 }>> = Mutex::new(Bitmap::new());
static B6: Mutex<Bitmap<{ 256 << 10 }>> = Mutex::new(Bitmap::new());
static B7: Mutex<Bitmap<{ 512 << 10 }>> = Mutex::new(Bitmap::new());
static B8: Mutex<Bitmap<{ 1 << 20 }>> = Mutex::new(Bitmap::new());
static B9: Mutex<Bitmap<{ 2 << 20 }>> = Mutex::new(Bitmap::new());
static BA: Mutex<Bitmap<{ 4 << 20 }>> = Mutex::new(Bitmap::new());
static BB: Mutex<Bitmap<{ 8 << 20 }>> = Mutex::new(Bitmap::new());
static BC: Mutex<Bitmap<{ 16 << 20 }>> = Mutex::new(Bitmap::new());
static BD: Mutex<Bitmap<{ 32 << 20 }>> = Mutex::new(Bitmap::new());
static BE: Mutex<Bitmap<{ 64 << 20 }>> = Mutex::new(Bitmap::new());
static BF: Mutex<Bitmap<{ 128 << 20 }>> = Mutex::new(Bitmap::new());

static OKAY: AtomicCell<bool> = AtomicCell::new(false);

pub fn init_global() {
    OKAY.compare_exchange(false, true).unwrap();
    let mut addr = VMM.heap_segment.start().to_usize();
    L1.lock().init(&mut addr);
    L2.lock().init(&mut addr);
    L3.lock().init(&mut addr);
    L4.lock().init(&mut addr);
    L5.lock().init(&mut addr);
    L6.lock().init(&mut addr);
    L7.lock().init(&mut addr);
    B0.lock().init(&mut addr);
    B1.lock().init(&mut addr);
    B2.lock().init(&mut addr);
    B3.lock().init(&mut addr);
    B4.lock().init(&mut addr);
    B5.lock().init(&mut addr);
    B6.lock().init(&mut addr);
    B7.lock().init(&mut addr);
    B8.lock().init(&mut addr);
    B9.lock().init(&mut addr);
    BA.lock().init(&mut addr);
    BB.lock().init(&mut addr);
    BC.lock().init(&mut addr);
    BD.lock().init(&mut addr);
    BE.lock().init(&mut addr);
    BF.lock().init(&mut addr);
    if addr > VMM.heap_segment.end().unwrap().to_usize() {
        panic!();
    }
}

pub struct DefaultAllocator;

unsafe impl Allocator for DefaultAllocator {
    #[allow(clippy::match_overlapping_arm)]
    fn allocate(&self, layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
        if !OKAY.load() {
            return Err(AllocError);
        }
        let layout = layout.pad_to_align();
        if layout.align() > 65536 {
            return Err(AllocError);
        }
        let data_address = match layout.size() {
            0 => NonNull::new(layout.align() as *mut ()).map(|x| x.cast()),
            1..=32 => L1.lock().alloc(),
            1..=64 => L2.lock().alloc(),
            1..=128 => L3.lock().alloc(),
            1..=256 => L4.lock().alloc(),
            1..=512 => L5.lock().alloc(),
            1..=1024 => L6.lock().alloc(),
            1..=2048 => L7.lock().alloc(),
            1..=0x1000 => B0.lock().alloc(),
            1..=0x2000 => B1.lock().alloc(),
            1..=0x4000 => B2.lock().alloc(),
            1..=0x8000 => B3.lock().alloc(),
            1..=0x10000 => B4.lock().alloc(),
            1..=0x20000 => B5.lock().alloc(),
            1..=0x40000 => B6.lock().alloc(),
            1..=0x80000 => B7.lock().alloc(),
            1..=0x100000 => B8.lock().alloc(),
            1..=0x200000 => B9.lock().alloc(),
            1..=0x400000 => BA.lock().alloc(),
            1..=0x800000 => BB.lock().alloc(),
            1..=0x1000000 => BC.lock().alloc(),
            1..=0x2000000 => BD.lock().alloc(),
            1..=0x4000000 => BE.lock().alloc(),
            1..=0x8000000 => BF.lock().alloc(),
            _ => None,
        }
        .ok_or(AllocError)?;
        Ok(NonNull::from_raw_parts(data_address.cast(), layout.size()))
    }

    #[allow(clippy::match_overlapping_arm)]
    unsafe fn deallocate(&self, ptr: NonNull<u8>, layout: Layout) {
        if !OKAY.load() {
            panic!();
        }
        let layout = layout.pad_to_align();
        assert!(layout.align() <= 65536);
        match layout.size() {
            0 => assert_eq!(ptr.as_ptr() as usize, layout.align()),
            1..=32 => L1.lock().dealloc(ptr),
            1..=64 => L2.lock().dealloc(ptr),
            1..=128 => L3.lock().dealloc(ptr),
            1..=256 => L4.lock().dealloc(ptr),
            1..=512 => L5.lock().dealloc(ptr),
            1..=1024 => L6.lock().dealloc(ptr),
            1..=2048 => L7.lock().dealloc(ptr),
            1..=0x1000 => B0.lock().dealloc(ptr),
            1..=0x2000 => B1.lock().dealloc(ptr),
            1..=0x4000 => B2.lock().dealloc(ptr),
            1..=0x8000 => B3.lock().dealloc(ptr),
            1..=0x10000 => B4.lock().dealloc(ptr),
            1..=0x20000 => B5.lock().dealloc(ptr),
            1..=0x40000 => B6.lock().dealloc(ptr),
            1..=0x80000 => B7.lock().dealloc(ptr),
            1..=0x100000 => B8.lock().dealloc(ptr),
            1..=0x200000 => B9.lock().dealloc(ptr),
            1..=0x400000 => BA.lock().dealloc(ptr),
            1..=0x800000 => BB.lock().dealloc(ptr),
            1..=0x1000000 => BC.lock().dealloc(ptr),
            1..=0x2000000 => BD.lock().dealloc(ptr),
            1..=0x4000000 => BE.lock().dealloc(ptr),
            1..=0x8000000 => BF.lock().dealloc(ptr),
            _ => (),
        }
    }
}
