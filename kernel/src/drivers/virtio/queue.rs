use super::mmio::MMIO;
use crate::prelude::*;
use core::sync::atomic::fence;
use core::sync::atomic::Ordering;
use mem::frames::PhysBox;
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
    desc: PhysBox<[ReadWrite<Desc>]>,
    avail: PhysBox<Avail>,
    used: PhysBox<Used>,
    desc_free: Vec<u16>,
    used_last: u16,
}

impl VirtQueue {
    pub fn new(mmio: &mut MMIO, index: u32, size: u16) -> Result<Self, VirtQueueError> {
        use VirtQueueError::*;
        mmio.queue_select(index);
        if mmio.queue_max_size() as u16 == 0 || !size.is_power_of_two() {
            return Err(NotAvailable);
        }
        let desc =
            unsafe { PhysBox::<[ReadWrite<Desc>]>::new_zeroed_unsize(size as usize).unwrap() };
        let avail = unsafe { PhysBox::<Avail>::new_zeroed_unsize(size as usize).unwrap() };
        let used = unsafe { PhysBox::<Used>::new_zeroed_unsize(size as usize).unwrap() };
        mmio.queue_init(
            size as u32,
            desc.paddr().to_usize() as u64,
            avail.paddr().to_usize() as u64,
            used.paddr().to_usize() as u64,
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
    pub fn push(
        &mut self,
        inputs: &[(PAddr, usize)],
        outputs: &[(PAddr, usize)],
    ) -> Result<u16, VirtQueueError> {
        unsafe {
            use VirtQueueError::*;
            let desc = &mut (*self.desc.get());
            let avail = &mut (*self.avail.get());
            let count = inputs.len() + outputs.len();
            if count == 0 {
                return Err(Underflow);
            }
            if count > self.desc_free.len() {
                return Err(Overflow);
            }
            let slots = self.desc_free.split_off(self.desc_free.len() - count);
            let mut p = 0;
            for &(paddr, len) in inputs {
                let next = slots.get(p + 1).cloned();
                desc[slots[p] as usize].write(Desc {
                    addr: paddr.to_usize() as u64,
                    len: len as u32,
                    flags: next.map_or(Desc::FLAGS_NONE, |_| Desc::FLAGS_NEXT),
                    next: next.unwrap_or(0),
                });
                p += 1;
            }
            for &(paddr, len) in outputs {
                let next = slots.get(p + 1).cloned();
                desc[slots[p] as usize].write(Desc {
                    addr: paddr.to_usize() as u64,
                    len: len as u32,
                    flags: next.map_or(Desc::FLAGS_NONE, |_| Desc::FLAGS_NEXT) | Desc::FLAGS_WRITE,
                    next: next.unwrap_or(0),
                });
                p += 1;
            }
            let avail_index = avail.idx.read();
            avail.ring[avail_index as usize].write(slots[0]);
            fence(Ordering::SeqCst);
            avail.idx.write((avail_index + 1) % self.size());
            Ok(slots[0])
        }
    }
    pub fn pop(&mut self) -> Option<(u16, usize)> {
        unsafe {
            let desc = &mut (*self.desc.get());
            let used = &mut (*self.used.get());
            let index = used.idx.read();
            if index == self.used_last {
                return None;
            }
            fence(Ordering::SeqCst);
            let (id, len) = (*self.used.get()).ring[self.used_last as usize].read();
            self.used_last = (self.used_last + 1) % self.size() as u16;
            let mut i = id as u16;
            loop {
                let d = desc[i as usize].read();
                self.desc_free.push(i);
                if d.flags & Desc::FLAGS_NEXT == Desc::FLAGS_NONE {
                    break;
                }
                i = d.next;
            }
            Some((id as u16, len as usize))
        }
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

#[repr(C, align(4))]
#[derive(Debug)]
pub struct Used {
    pub flags: ReadWrite<u16>,
    pub idx: ReadWrite<u16>,
    pub ring: [ReadWrite<(u32, u32)>],
}
