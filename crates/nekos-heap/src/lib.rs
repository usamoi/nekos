#![no_std]
#![feature(int_log)]

pub mod fallback;
pub mod slab;
pub mod unit;

pub trait Mmap {
    fn map(vaddr: usize);
    fn unmap(vaddr: usize);
}
