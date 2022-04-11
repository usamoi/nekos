use crate::prelude::*;
use core::alloc::Layout;
use core::fmt::{Debug, Formatter, Pointer};
use core::ops::{Add, Sub};

// linker symbol

extern "C" {
    pub type LinkerSymbol;
    pub static _kernel_address: LinkerSymbol;
}

impl LinkerSymbol {
    pub fn as_usize(&self) -> usize {
        self as *const _ as usize
    }
    pub fn as_vaddr(&self) -> VAddr {
        VAddr::new(self.as_usize())
    }
    pub fn as_paddr(&self) -> PAddr {
        unsafe { config::KERNEL_ADDRESS + (self.as_vaddr() - _kernel_address.as_vaddr()) }
    }
    pub const fn as_ptr<T>(&self) -> *const T {
        self as *const _ as *const T
    }
    pub const fn as_mut_ptr<T>(&self) -> *mut T {
        self as *const _ as *mut T
    }
    pub fn size_between(&self, other: &LinkerSymbol) -> usize {
        unsafe { other.as_ptr::<u8>().offset_from(self.as_ptr::<u8>()) as usize }
    }
}

impl Pointer for LinkerSymbol {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:#x}", self.as_usize())
    }
}

impl Debug for LinkerSymbol {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:#x}", self.as_usize())
    }
}

// address

#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, derive_more::Pointer)]
#[pointer(fmt = "{:#x}", _0)]
pub struct PAddr(usize);

impl PAddr {
    pub const NULL: Self = PAddr::new(0);
    pub const fn new(x: usize) -> Self {
        Self(x)
    }
    pub const fn to_usize(self) -> usize {
        self.0
    }
    pub const fn to_const(self) -> *const u8 {
        self.0 as *const _
    }
    pub const fn to_mut(self) -> *mut u8 {
        self.0 as *mut _
    }
    pub const fn align_to(self, x: usize) -> Self {
        Self(self.0.next_multiple_of(x))
    }
}

impl From<PAddr> for usize {
    fn from(x: PAddr) -> usize {
        x.0
    }
}

impl From<usize> for PAddr {
    fn from(x: usize) -> PAddr {
        PAddr(x)
    }
}

impl const Add<usize> for PAddr {
    type Output = Self;

    fn add(self, rhs: usize) -> Self::Output {
        Self(self.0.wrapping_add(rhs))
    }
}

impl Sub<usize> for PAddr {
    type Output = Self;

    fn sub(self, rhs: usize) -> Self::Output {
        Self(self.0.wrapping_sub(rhs))
    }
}

impl Sub<Self> for PAddr {
    type Output = usize;

    fn sub(self, rhs: Self) -> Self::Output {
        self.0.wrapping_sub(rhs.0)
    }
}

impl Debug for PAddr {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        Pointer::fmt(self, f)
    }
}

#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, derive_more::Pointer)]
#[pointer(fmt = "{:#x}", _0)]
pub struct VAddr(usize);

impl VAddr {
    pub const fn new(x: usize) -> Self {
        Self(x)
    }
    pub const fn to_usize(self) -> usize {
        self.into()
    }
}

impl From<VAddr> for usize {
    fn from(x: VAddr) -> usize {
        x.0
    }
}

impl From<usize> for VAddr {
    fn from(x: usize) -> VAddr {
        VAddr(x)
    }
}

impl const Add<usize> for VAddr {
    type Output = Self;

    fn add(self, rhs: usize) -> Self::Output {
        Self(self.0.wrapping_add(rhs))
    }
}

impl Sub<usize> for VAddr {
    type Output = Self;

    fn sub(self, rhs: usize) -> Self::Output {
        Self(self.0.wrapping_sub(rhs))
    }
}

impl Sub<Self> for VAddr {
    type Output = usize;

    fn sub(self, rhs: Self) -> Self::Output {
        self.0.wrapping_sub(rhs.0)
    }
}

impl Debug for VAddr {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        Pointer::fmt(self, f)
    }
}

// segment

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Segment<P> {
    start: P,
    end: Option<P>,
}

impl<P: Copy> Segment<P> {
    pub const fn start(self) -> P {
        self.start
    }
    pub const fn end(self) -> Option<P> {
        self.end
    }
}

impl<P> Segment<P> {
    pub const fn new(start: P, end: Option<P>) -> Option<Self>
    where
        P: ~const InSegment,
    {
        InSegment::new(start, end)
    }
}

impl<P: InSegment> Segment<P> {
    pub const fn wrapping_end(self) -> P {
        match self.end {
            Some(end) => end,
            None => P::ZERO,
        }
    }
    pub fn is_empty(self) -> bool {
        match self.end {
            Some(end) => (self.start..end).is_empty(),
            None => false,
        }
    }
    pub fn contains(self, other: impl SegmentContains<P>) -> bool {
        SegmentContains::contains(self, other)
    }
}

impl<P: Debug> Debug for Segment<P> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match &self.end {
            Some(end) => write!(f, "{:?}..{:?}", self.start, end),
            None => write!(f, "{:?}..", self.start),
        }
    }
}

pub trait InSegment: Copy + Ord {
    const ZERO: Self;
    fn new(start: Self, end: Option<Self>) -> Option<Segment<Self>>;
    fn forward(self, x: usize) -> Option<Option<Self>>;
}

impl const InSegment for usize {
    const ZERO: Self = 0;
    fn new(start: Self, end: Option<Self>) -> Option<Segment<Self>> {
        match end {
            Some(end) if start > end => None,
            end => Some(Segment { start, end }),
        }
    }
    fn forward(self, x: usize) -> Option<Option<Self>> {
        if self == 0 || self.wrapping_neg() > x {
            return Some(Some(self + x));
        }
        if self.wrapping_neg() == x {
            return Some(None);
        }
        None
    }
}

impl const InSegment for PAddr {
    const ZERO: Self = Self::new(0);
    fn new(start: Self, end: Option<Self>) -> Option<Segment<Self>> {
        match end {
            Some(end) if start.0 > end.0 => None,
            end => Some(Segment { start, end }),
        }
    }
    fn forward(self, x: usize) -> Option<Option<Self>> {
        if self.0 == 0 || self.0.wrapping_neg() > x {
            return Some(Some(self + x));
        }
        if self.0.wrapping_neg() == x {
            return Some(None);
        }
        None
    }
}

impl const InSegment for VAddr {
    const ZERO: Self = Self::new(0);
    fn new(start: Self, end: Option<Self>) -> Option<Segment<Self>> {
        match end {
            Some(end) if start.0 > end.0 => None,
            end => Some(Segment { start, end }),
        }
    }
    fn forward(self, x: usize) -> Option<Option<Self>> {
        if self.0 == 0 || self.0.wrapping_neg() > x {
            return Some(Some(self + x));
        }
        if self.0.wrapping_neg() == x {
            return Some(None);
        }
        None
    }
}

pub trait SegmentContains<P> {
    fn contains(lhs: Segment<P>, rhs: Self) -> bool;
}

impl<P: InSegment> SegmentContains<P> for P {
    fn contains(lhs: Segment<P>, rhs: Self) -> bool {
        match lhs.end {
            Some(r) => lhs.start <= rhs && rhs < r,
            None => lhs.start <= rhs,
        }
    }
}

impl<P: InSegment> SegmentContains<P> for Segment<P> {
    fn contains(lhs: Segment<P>, rhs: Self) -> bool {
        if rhs.is_empty() {
            return true;
        }
        if lhs.is_empty() {
            return false;
        }
        match (lhs.end, rhs.end) {
            (Some(le), Some(re)) => lhs.start <= rhs.start && re <= le,
            (Some(_), None) => false,
            (None, _) => lhs.start <= rhs.start,
        }
    }
}

pub const fn by_size<T: ~const InSegment>(addr: T, size: usize) -> Option<Segment<T>> {
    Segment::new(addr, addr.forward(size)?)
}

pub const fn by_points<T: ~const InSegment>(start: T, end: T) -> Option<Segment<T>> {
    Segment::new(start, Some(end))
}

// memory

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Access {
    Instruction,
    Load,
    Store,
}

// map

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MapLayout {
    size: usize,
    align: usize,
}

impl MapLayout {
    pub const fn new(size: usize, align: usize) -> Option<Self> {
        if align.is_power_of_two() && size % align == 0 {
            Some(Self { size, align })
        } else {
            None
        }
    }
    pub const fn size(self) -> usize {
        self.size
    }
    pub const fn align(self) -> usize {
        self.align
    }
    pub const fn check(self, addr: usize) -> bool {
        let check_align = addr & (self.align - 1) == 0;
        let check_size = addr.wrapping_add(self.size) == 0 || addr.checked_add(self.size).is_some();
        check_align && check_size
    }
    pub const fn from_rust(layout: Layout) -> Self {
        let size = layout.size().next_multiple_of(layout.align());
        Self::new(size, layout.align()).unwrap()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Permission {
    pub read: bool,
    pub write: bool,
    pub execute: bool,
}

impl Permission {
    pub const RO: Self = Self::new(true, false, false);
    pub const RW: Self = Self::new(true, true, false);
    pub const EO: Self = Self::new(false, false, true);
    pub const fn new(read: bool, write: bool, execute: bool) -> Self {
        Self {
            read,
            write,
            execute,
        }
    }
}

impl const From<Permission> for u8 {
    fn from(x: Permission) -> u8 {
        x.read as u8 | (x.write as u8) << 1 | (x.execute as u8) << 2
    }
}

pub trait Map {
    fn layout(&self) -> MapLayout;
    fn len(&self) -> usize {
        self.layout().size() / self.layout().align()
    }
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

pub trait MapRead: Map {
    unsafe fn read_unchecked(&self, offset: usize, buffer: &mut [u8]);

    fn read(&self, offset: usize, buffer: &mut [u8]) {
        if self.layout().size() < offset + buffer.len() {
            panic!(
                "the size is {} but the expected size is {}",
                self.layout().size(),
                offset + buffer.len()
            );
        }
        unsafe { self.read_unchecked(offset, buffer) }
    }
}

pub trait MapWrite: Map {
    unsafe fn write_unchecked(&self, offset: usize, buffer: &[u8]);

    fn write(&self, offset: usize, buffer: &[u8]) {
        if self.layout().size() < offset + buffer.len() {
            panic!(
                "the size is {} but the expected size is {}",
                self.layout().size(),
                offset + buffer.len()
            );
        }
        unsafe { self.write_unchecked(offset, buffer) }
    }
}

pub trait MapIndex: Map {
    unsafe fn index_unchecked(&self, i: usize) -> PAddr;
    fn index(&self, i: usize) -> PAddr {
        if self.len() <= i {
            panic!("the len is {} but the index is {}", self.len(), i);
        }
        unsafe { self.index_unchecked(i) }
    }
}
