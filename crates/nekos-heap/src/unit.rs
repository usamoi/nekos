use crate::Mmap;
use core::marker::PhantomData;
use core::ptr::NonNull;

// small buffers, 32 <= S < 4096, T = S * 65536 / 4096
pub struct UnitA<const S: usize, const T: usize, M> {
    addr: usize,
    count: [u8; T],
    next: [u16; 65536],
    head: Option<u16>,
    _maker: PhantomData<fn(M) -> M>,
}

impl<const S: usize, const T: usize, M: Mmap> UnitA<S, T, M> {
    pub fn new(addr: &mut usize) -> Self {
        let start = *addr;
        *addr += S * 65536;
        Self {
            addr: start,
            count: [0u8; T],
            next: {
                let mut next = [0u16; 65536];
                for i in 0..65535 {
                    next[i as usize] = i + 1;
                }
                next[65535] = 65535;
                next
            },
            head: Some(0),
            _maker: PhantomData,
        }
    }
    pub fn test(&self, addr: NonNull<u8>) -> bool {
        let addr = addr.as_ptr() as usize;
        self.addr <= addr && addr < self.addr + S * 65536
    }
    pub fn alloc(&mut self) -> Option<NonNull<u8>> {
        let x = self.head.take()?;
        if self.next[x as usize] != x {
            self.head = Some(self.next[x as usize]);
        }
        // reference count
        let page_0 = (x as usize * S) / 4096;
        if self.count[page_0] == 0 {
            M::map(self.addr + page_0 * 4096);
        }
        self.count[page_0] += 1;
        let page_1 = (x as usize * S + S - 1) / 4096;
        if page_1 != page_0 {
            if self.count[page_1] == 0 {
                M::map(self.addr + page_1 * 4096);
            }
            self.count[page_1] += 1;
        }
        //
        Some(NonNull::new((self.addr + (x as usize) * S) as *mut u8).unwrap())
    }
    pub fn dealloc(&mut self, addr: NonNull<u8>) {
        let x = ((addr.as_ptr() as usize - self.addr) / S) as u16;
        self.next[x as usize] = self.head.take().unwrap_or(x);
        self.head = Some(x);
        // reference count
        let page_0 = (x as usize * S) / 4096;
        self.count[page_0] -= 1;
        if self.count[page_0] == 0 {
            M::unmap(self.addr + page_0 * 4096);
        }
        let page_1 = (x as usize * S + S - 1) / 4096;
        if page_1 != page_0 {
            self.count[page_1] -= 1;
            if self.count[page_1] == 0 {
                M::unmap(self.addr + page_1 * 4096);
            }
        }
    }
}

// big buffers, S = 4096k
pub struct UnitB<const S: usize, M> {
    addr: usize,
    bits: u64,
    _maker: PhantomData<fn(M) -> M>,
}

impl<const S: usize, M: Mmap> UnitB<S, M> {
    pub fn new(addr: &mut usize) -> Self {
        let start = *addr;
        *addr += S * 64;
        Self {
            addr: start,
            bits: u64::MAX,
            _maker: PhantomData,
        }
    }
    pub fn is_full(&self) -> bool {
        self.bits == u64::MAX
    }
    pub fn is_empty(&self) -> bool {
        self.bits == 0
    }
    pub fn test(&self, addr: NonNull<u8>) -> bool {
        let addr = addr.as_ptr() as usize;
        self.addr <= addr && addr < self.addr + S * 64
    }
    pub fn alloc(&mut self) -> Option<NonNull<u8>> {
        let x = self.bits.checked_log2()?;
        self.bits ^= 1 << x;
        for i in 0..(S / 4096) {
            M::map(self.addr + (x as usize) * S + i * 4096);
        }
        Some(NonNull::new((self.addr + x as usize * S) as *mut u8).unwrap())
    }
    pub fn dealloc(&mut self, addr: NonNull<u8>) {
        let x = (addr.as_ptr() as usize - self.addr) / S;
        self.bits ^= 1 << x;
        for i in 0..(S / 4096) {
            M::unmap(self.addr + (x as usize) * S + i * 4096);
        }
    }
}
