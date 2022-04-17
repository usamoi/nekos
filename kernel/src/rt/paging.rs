use crate::prelude::*;
use core::fmt::Debug;

#[derive(Debug)]
pub enum PagingMapError {
    InvalidVAddr,
    InvalidPAddr,
    AlignNotSupported,
    PermissionNotSupported,
}

#[derive(Debug)]
pub enum PagingUnmapError {
    InvalidVAddr,
    AlignNotSupported,
}

pub trait Paging: Debug + Send + Sync {
    fn new() -> Self;
    fn map(
        &self,
        vaddr: VAddr,
        paddr: PAddr,
        align: usize,
        permission: Permission,
        user: bool,
        global: bool,
    ) -> Result<(), PagingMapError>;
    fn unmap(&self, vaddr: VAddr, align: usize) -> Result<PAddr, PagingUnmapError>;
}
