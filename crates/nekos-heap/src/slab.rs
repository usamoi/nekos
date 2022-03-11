use crate::unit::{UnitA, UnitB};
use crate::Mmap;
use core::alloc::Layout;
use core::ptr::NonNull;

pub type U32<M> = UnitA<32, { 32 * 65536 / 4096 }, M>;
pub type U64<M> = UnitA<64, { 64 * 65536 / 4096 }, M>;
pub type U128<M> = UnitA<128, { 128 * 65536 / 4096 }, M>;
pub type U256<M> = UnitA<256, { 256 * 65536 / 4096 }, M>;
pub type U512<M> = UnitA<512, { 512 * 65536 / 4096 }, M>;
pub type U1024<M> = UnitA<1024, { 1024 * 65536 / 4096 }, M>;
pub type U2048<M> = UnitA<2048, { 2048 * 65536 / 4096 }, M>;
pub type U4K<M> = UnitB<{ 4 << 10 }, M>; // 4 KB
pub type U8K<M> = UnitB<{ 8 << 10 }, M>; // 8 KB
pub type U16K<M> = UnitB<{ 16 << 10 }, M>; // 16 KB
pub type U32K<M> = UnitB<{ 32 << 10 }, M>; // 32 KB
pub type U64K<M> = UnitB<{ 64 << 10 }, M>; // 64 KB
pub type U128K<M> = UnitB<{ 128 << 10 }, M>; // 128 KB
pub type U256K<M> = UnitB<{ 256 << 10 }, M>; // 256 KB
pub type U512K<M> = UnitB<{ 512 << 10 }, M>; // 512 KB
pub type U1M<M> = UnitB<{ 1 << 20 }, M>; // 1 MB
pub type U2M<M> = UnitB<{ 2 << 20 }, M>; // 2 MB
pub type U4M<M> = UnitB<{ 4 << 20 }, M>; // 4 MB
pub type U8M<M> = UnitB<{ 8 << 20 }, M>; // 8 MB
pub type U16M<M> = UnitB<{ 16 << 20 }, M>; // 16 MB
pub type U32M<M> = UnitB<{ 32 << 20 }, M>; // 32 MB
pub type U64M<M> = UnitB<{ 64 << 20 }, M>; // 64 MB
pub type U128M<M> = UnitB<{ 128 << 20 }, M>; // 128 MB

struct HeapA<M: Mmap> {
    b32: U32<M>,
    b64: U64<M>,
    b128: U128<M>,
    b256: U256<M>,
    b512: U512<M>,
    b1024: U1024<M>,
    b2048: U2048<M>,
}

struct HeapB<M: Mmap> {
    k4: U4K<M>,
    k8: U8K<M>,
    k16: U16K<M>,
    k32: U32K<M>,
    k64: U64K<M>,
    k128: U128K<M>,
    k256: U256K<M>,
    k512: U512K<M>,
    m1: U1M<M>,
    m2: U2M<M>,
    m4: U4M<M>,
    m8: U8M<M>,
    m16: U16M<M>,
    m32: U32M<M>,
    m64: U64M<M>,
    m128: U128M<M>,
}

pub struct Heap<M: Mmap> {
    start: usize,
    end: usize,
    a: HeapA<M>,
    b: HeapB<M>,
}

impl<M: Mmap> Heap<M> {
    pub fn new(start: usize, end: usize) -> Self {
        let mut addr = start;
        let b32 = U32::new(&mut addr);
        let b64 = U64::new(&mut addr);
        let b128 = U128::new(&mut addr);
        let b256 = U256::new(&mut addr);
        let b512 = U512::new(&mut addr);
        let b1024 = U1024::new(&mut addr);
        let b2048 = U2048::new(&mut addr);
        let k4 = U4K::new(&mut addr);
        let k8 = U8K::new(&mut addr);
        let k16 = U16K::new(&mut addr);
        let k32 = U32K::new(&mut addr);
        let k64 = U64K::new(&mut addr);
        let k128 = U128K::new(&mut addr);
        let k256 = U256K::new(&mut addr);
        let k512 = U512K::new(&mut addr);
        let m1 = U1M::new(&mut addr);
        let m2 = U2M::new(&mut addr);
        let m4 = U4M::new(&mut addr);
        let m8 = U8M::new(&mut addr);
        let m16 = U16M::new(&mut addr);
        let m32 = U32M::new(&mut addr);
        let m64 = U64M::new(&mut addr);
        let m128 = U128M::new(&mut addr);
        if addr > end {
            panic!();
        }
        Self {
            start,
            end,
            a: HeapA {
                b32,
                b64,
                b128,
                b256,
                b512,
                b1024,
                b2048,
            },
            b: HeapB {
                k4,
                k8,
                k16,
                k32,
                k64,
                k128,
                k256,
                k512,
                m1,
                m2,
                m4,
                m8,
                m16,
                m32,
                m64,
                m128,
            },
        }
    }
    pub fn test(&self, addr: NonNull<u8>) -> bool {
        let addr = addr.as_ptr() as usize;
        self.start <= addr && addr < self.end
    }
    pub fn alloc(&mut self, layout: Layout) -> Option<NonNull<u8>> {
        let layout = layout.pad_to_align();
        if layout.align() > 65536 {
            return None;
        }
        match layout.size() {
            0..=0x20 => {
                if let Some(addr) = self.a.b32.alloc() {
                    return Some(addr);
                }
                None
            }
            0..=0x40 => {
                if let Some(addr) = self.a.b64.alloc() {
                    return Some(addr);
                }
                None
            }
            0..=0x80 => {
                if let Some(addr) = self.a.b128.alloc() {
                    return Some(addr);
                }
                None
            }
            0..=0x100 => {
                if let Some(addr) = self.a.b256.alloc() {
                    return Some(addr);
                }
                None
            }
            0..=0x200 => {
                if let Some(addr) = self.a.b512.alloc() {
                    return Some(addr);
                }
                None
            }
            0..=0x400 => {
                if let Some(addr) = self.a.b1024.alloc() {
                    return Some(addr);
                }
                None
            }
            0..=0x800 => {
                if let Some(addr) = self.a.b2048.alloc() {
                    return Some(addr);
                }
                None
            }
            0..=0x1000 => {
                if let Some(addr) = self.b.k4.alloc() {
                    return Some(addr);
                }
                None
            }
            0..=0x2000 => {
                if let Some(addr) = self.b.k8.alloc() {
                    return Some(addr);
                }
                None
            }
            0..=0x4000 => {
                if let Some(addr) = self.b.k16.alloc() {
                    return Some(addr);
                }
                None
            }
            0..=0x8000 => {
                if let Some(addr) = self.b.k32.alloc() {
                    return Some(addr);
                }
                None
            }
            0..=0x10000 => {
                if let Some(addr) = self.b.k64.alloc() {
                    return Some(addr);
                }
                None
            }
            0..=0x20000 => {
                if let Some(addr) = self.b.k128.alloc() {
                    return Some(addr);
                }
                None
            }
            0..=0x40000 => {
                if let Some(addr) = self.b.k256.alloc() {
                    return Some(addr);
                }
                None
            }
            0..=0x80000 => {
                if let Some(addr) = self.b.k512.alloc() {
                    return Some(addr);
                }
                None
            }
            0..=0x100000 => {
                if let Some(addr) = self.b.m1.alloc() {
                    return Some(addr);
                }
                None
            }
            0..=0x200000 => {
                if let Some(addr) = self.b.m2.alloc() {
                    return Some(addr);
                }
                None
            }
            0..=0x400000 => {
                if let Some(addr) = self.b.m4.alloc() {
                    return Some(addr);
                }
                None
            }
            0..=0x800000 => {
                if let Some(addr) = self.b.m8.alloc() {
                    return Some(addr);
                }
                None
            }
            0..=0x1000000 => {
                if let Some(addr) = self.b.m16.alloc() {
                    return Some(addr);
                }
                None
            }
            0..=0x2000000 => {
                if let Some(addr) = self.b.m32.alloc() {
                    return Some(addr);
                }
                None
            }
            0..=0x4000000 => {
                if let Some(addr) = self.b.m64.alloc() {
                    return Some(addr);
                }
                None
            }
            0..=0x8000000 => {
                if let Some(addr) = self.b.m128.alloc() {
                    return Some(addr);
                }
                None
            }
            _ => None,
        }
    }
    pub fn dealloc(&mut self, addr: NonNull<u8>, layout: Layout) {
        let layout = layout.pad_to_align();
        if layout.size() > 65536 {
            panic!();
        }
        match layout.size() {
            0..=0x20 => {
                if self.a.b32.test(addr) {
                    self.a.b32.dealloc(addr);
                    return;
                }
                panic!();
            }
            0..=0x40 => {
                if self.a.b64.test(addr) {
                    self.a.b64.dealloc(addr);
                    return;
                }
                panic!();
            }
            0..=0x80 => {
                if self.a.b128.test(addr) {
                    self.a.b128.dealloc(addr);
                    return;
                }
                panic!();
            }
            0..=0x100 => {
                if self.a.b256.test(addr) {
                    self.a.b256.dealloc(addr);
                    return;
                }
                panic!();
            }
            0..=0x200 => {
                if self.a.b512.test(addr) {
                    self.a.b512.dealloc(addr);
                    return;
                }
                panic!();
            }
            0..=0x400 => {
                if self.a.b1024.test(addr) {
                    self.a.b1024.dealloc(addr);
                    return;
                }
                panic!();
            }
            0..=0x800 => {
                if self.a.b2048.test(addr) {
                    self.a.b2048.dealloc(addr);
                    return;
                }
                panic!();
            }
            0..=0x1000 => {
                if self.b.k4.test(addr) {
                    self.b.k4.dealloc(addr);
                    return;
                }
                panic!();
            }
            0..=0x2000 => {
                if self.b.k8.test(addr) {
                    self.b.k8.dealloc(addr);
                    return;
                }
                panic!();
            }
            0..=0x4000 => {
                if self.b.k16.test(addr) {
                    self.b.k16.dealloc(addr);
                    return;
                }
                panic!();
            }
            0..=0x8000 => {
                if self.b.k32.test(addr) {
                    self.b.k32.dealloc(addr);
                    return;
                }
                panic!();
            }
            0..=0x10000 => {
                if self.b.k64.test(addr) {
                    self.b.k64.dealloc(addr);
                    return;
                }
                panic!();
            }
            0..=0x20000 => {
                if self.b.k128.test(addr) {
                    self.b.k128.dealloc(addr);
                    return;
                }
                panic!();
            }
            0..=0x40000 => {
                if self.b.k256.test(addr) {
                    self.b.k256.dealloc(addr);
                    return;
                }
                panic!();
            }
            0..=0x80000 => {
                if self.b.k512.test(addr) {
                    self.b.k512.dealloc(addr);
                    return;
                }
                panic!();
            }
            0..=0x100000 => {
                if self.b.m1.test(addr) {
                    self.b.m1.dealloc(addr);
                    return;
                }
                panic!();
            }
            0..=0x200000 => {
                if self.b.m2.test(addr) {
                    self.b.m2.dealloc(addr);
                    return;
                }
                panic!();
            }
            0..=0x400000 => {
                if self.b.m4.test(addr) {
                    self.b.m4.dealloc(addr);
                    return;
                }
                panic!();
            }
            0..=0x800000 => {
                if self.b.m8.test(addr) {
                    self.b.m8.dealloc(addr);
                    return;
                }
                panic!();
            }
            0..=0x1000000 => {
                if self.b.m16.test(addr) {
                    self.b.m16.dealloc(addr);
                    return;
                }
                panic!();
            }
            0..=0x2000000 => {
                if self.b.m32.test(addr) {
                    self.b.m32.dealloc(addr);
                    return;
                }
                panic!();
            }
            0..=0x4000000 => {
                if self.b.m64.test(addr) {
                    self.b.m64.dealloc(addr);
                    return;
                }
                panic!();
            }
            0..=0x8000000 => {
                if self.b.m128.test(addr) {
                    self.b.m128.dealloc(addr);
                    return;
                }
                panic!();
            }
            _ => (),
        }
    }
}
