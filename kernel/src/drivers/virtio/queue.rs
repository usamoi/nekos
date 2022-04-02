use super::mmio::MMIO;
use crate::prelude::*;
use core::alloc::Allocator;
use core::ptr::Pointee;
use core::sync::atomic::fence;
use core::sync::atomic::Ordering;
use mem::dma::{DmaAllocator, DmaBox};
use volatile::ReadOnly;
use volatile::ReadWrite;

// todo: endian

#[derive(Debug)]
pub enum VirtQueueError {
    NotAvailable,
    Underflow,
    Overflow,
}

#[repr(C)]
pub struct VirtQueue {
    index: u32,
    size: u16,
    desc: DmaBox<[ReadWrite<Desc>]>,
    avail: DmaBox<Avail>,
    used: DmaBox<Used>,
    desc_free: Vec<u16>,
    used_last: u16,
}

unsafe fn new_zeroed_unsize<T: ?Sized>(metadata: <T as Pointee>::Metadata) -> DmaBox<T> {
    let nullptr = core::ptr::from_raw_parts::<T>(core::ptr::null(), metadata);
    let layout = core::alloc::Layout::for_value_raw(nullptr);
    let nonnull = DmaAllocator.allocate_zeroed(layout).unwrap();
    let thin = nonnull.as_ptr() as *mut ();
    Box::from_raw_in(core::ptr::from_raw_parts_mut(thin, metadata), DmaAllocator)
}

impl VirtQueue {
    pub fn new(mmio: &mut MMIO, index: u32, size: u16) -> Result<Self, VirtQueueError> {
        use VirtQueueError::*;
        mmio.queue_select(index);
        if mmio.queue_max_size() as u16 == 0 || !size.is_power_of_two() {
            return Err(NotAvailable);
        }
        let desc: DmaBox<[ReadWrite<Desc>]> = unsafe { new_zeroed_unsize(size as usize) };
        let mut avail: DmaBox<Avail> = unsafe { new_zeroed_unsize(size as usize) };
        let mut used: DmaBox<Used> = unsafe { new_zeroed_unsize(size as usize) };
        mmio.queue_init(
            size as u32,
            desc.as_ptr() as u64,
            avail.as_mut() as *mut _ as *mut () as u64,
            used.as_mut() as *mut _ as *mut () as u64,
        );
        Ok(Self {
            index,
            size,
            desc,
            avail,
            used,
            desc_free: (0..size).collect(),
            used_last: 0,
        })
    }
    pub fn index(&self) -> u32 {
        self.index
    }
    pub fn size(&self) -> u16 {
        self.size
    }
    pub fn push(&mut self, refs: &[RawDmaRef], muts: &[RawDmaMut]) -> Result<u16, VirtQueueError> {
        use VirtQueueError::*;
        let count = refs.len() + muts.len();
        if count == 0 {
            return Err(Underflow);
        }
        if count > self.desc_free.len() {
            return Err(Overflow);
        }
        let slots = self.desc_free.split_off(self.desc_free.len() - count);
        let mut p = 0;
        for RawDmaRef(paddr, size) in refs.iter().cloned() {
            let flag_next = (p != count - 1)
                .then_some(Desc::FLAGS_NEXT)
                .unwrap_or_default();
            self.desc[slots[p] as usize].write(Desc {
                addr: paddr.to_usize() as u64,
                len: size as u32,
                flags: flag_next,
                next: slots.get(p + 1).cloned().unwrap_or(0),
            });
            dbg!(
                p,
                slots[p],
                Desc {
                    addr: paddr.to_usize() as u64,
                    len: size as u32,
                    flags: flag_next,
                    next: slots.get(p + 1).cloned().unwrap_or(0),
                }
            );
            p += 1;
        }
        for RawDmaMut(paddr, size) in muts.iter().cloned() {
            let flag_next = (p != count - 1)
                .then_some(Desc::FLAGS_NEXT)
                .unwrap_or_default();
            self.desc[slots[p] as usize].write(Desc {
                addr: paddr.to_usize() as u64,
                len: size as u32,
                flags: flag_next | Desc::FLAGS_WRITE,
                next: slots.get(p + 1).cloned().unwrap_or(0),
            });
            dbg!(
                p,
                slots[p],
                Desc {
                    addr: paddr.to_usize() as u64,
                    len: size as u32,
                    flags: flag_next | Desc::FLAGS_WRITE,
                    next: slots.get(p + 1).cloned().unwrap_or(0),
                }
            );
            p += 1;
        }
        self.avail.push(slots[0]);
        Ok(slots[0])
    }
    pub fn pop(&mut self) -> Option<(u16, u32)> {
        let (id, len) = self.used.pop(&mut self.used_last)?;
        let mut i = id as u16;
        loop {
            let d = self.desc[i as usize].read();
            self.desc_free.push(i);
            if d.flags & Desc::FLAGS_NEXT == Desc::FLAGS_NONE {
                break;
            }
            i = d.next;
        }
        Some((id, len))
    }
}

#[repr(C, align(16))]
#[derive(Debug, Clone, Copy)]
pub struct Desc {
    pub addr: u64,
    pub len: u32,
    pub flags: u16,
    pub next: u16,
}

impl Desc {
    pub const FLAGS_NONE: u16 = 0;
    pub const FLAGS_NEXT: u16 = 1;
    pub const FLAGS_WRITE: u16 = 2;
    pub const FLAGS_INDIRECT: u16 = 4;
}

#[repr(C, align(2))]
#[derive(Debug)]
pub struct Avail {
    pub flags: ReadWrite<u16>,
    pub idx: ReadWrite<u16>,
    pub ring: [ReadWrite<u16>],
}

impl Avail {
    pub fn push(&mut self, x: u16) {
        let avail_index = self.idx.read();
        self.ring[avail_index as usize & (self.ring.len() - 1)].write(x);
        fence(Ordering::SeqCst);
        self.idx.write(avail_index + 1);
    }
}

#[repr(C, align(4))]
#[derive(Debug)]
pub struct Used {
    pub flags: ReadWrite<u16>,
    pub idx: ReadOnly<u16>,
    pub ring: [ReadOnly<(u16, u16, u32)>],
}

impl Used {
    pub fn pop(&self, last: &mut u16) -> Option<(u16, u32)> {
        if *last == self.idx.read() {
            return None;
        }
        fence(Ordering::SeqCst);
        let (id, _, len) = self.ring[*last as usize].read();
        *last = (*last + 1) & (self.ring.len() as u16 - 1);
        Some((id as u16, len))
    }
}
