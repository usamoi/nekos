use crate::{mem::frames::PhysBox, prelude::*};
use volatile::ReadWrite;

// todo: endian

#[repr(C)]
pub struct VirtQueue<const N: usize> {
    desc: PhysBox<[Desc; N]>,
    avail: PhysBox<Avail<N>>,
    used: PhysBox<Used<N>>,
}

#[repr(C, align(16))]
pub struct Desc {
    pub addr: ReadWrite<u64>,
    pub len: ReadWrite<u32>,
    pub flags: ReadWrite<DescFlags>,
    pub next: ReadWrite<u16>,
}

impl Desc {
    pub fn has_next(&self) -> bool {
        self.flags.read() & DescFlags::NEXT != DescFlags::NONE
    }
    pub fn is_writable(&self) -> bool {
        self.flags.read() & DescFlags::WRITE != DescFlags::NONE
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, BitAnd, BitOr, BitXor)]
pub struct DescFlags(pub u16);

impl DescFlags {
    pub const NONE: Self = Self(0);
    pub const NEXT: Self = Self(1 << 0);
    pub const WRITE: Self = Self(1 << 1);
    pub const INDIRECT: Self = Self(1 << 2);
}

#[repr(C, align(2))]
#[derive(Debug)]
pub struct Avail<const N: usize> {
    pub flags: ReadWrite<u16>,
    pub idx: ReadWrite<u16>,
    pub ring: [ReadWrite<u16>; N],
}

#[repr(C, align(4))]
#[derive(Debug)]
pub struct Used<const N: usize> {
    pub flags: ReadWrite<u16>,
    pub idx: ReadWrite<u16>,
    pub ring: [UsedElem; N],
}

#[repr(C)]
#[derive(Debug)]
pub struct UsedElem {
    pub id: ReadWrite<u32>,
    pub len: ReadWrite<u32>,
}
