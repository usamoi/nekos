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

#[repr(transparent)]
#[derive(Debug, Clone, Copy)]
pub struct PagingToken(pub usize);

impl PagingToken {
    pub unsafe fn switch(self) {
        P::paging_token_switch(self);
    }
}

pub trait PagingGroup: Debug + Send + Sync {
    fn new() -> Self;
}

pub trait Paging: Debug + Send + Sync {
    type Group;
    fn new(group: &Self::Group) -> Self;
    fn token(&self) -> PagingToken;
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
