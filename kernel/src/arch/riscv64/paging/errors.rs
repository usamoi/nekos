use crate::prelude::*;

#[derive(Debug)]
pub(super) enum InnerError {
    Overlapping,
}

#[derive(Debug)]
pub enum PageTableMapError {
    InvalidVAddr,
    InvalidPAddr,
    AlignNotSupported,
    PermissionNotSupported,
    Overlapping,
}

fully!(InnerError, PageTableMapError; Overlapping);

#[derive(Debug)]
pub enum PageTableUnmapError {
    InvalidVAddr,
    AlignNotSupported,
    Overlapping,
}

fully!(InnerError, PageTableUnmapError; Overlapping);
