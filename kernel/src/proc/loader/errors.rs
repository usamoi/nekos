use crate::prelude::*;
use proc::vmm::*;
use user::objects::memory::*;
use zelf::program::{ParseProgramError, ParseProgramsError};

#[derive(Debug)]
pub enum LoadError {
    NotFound,
    BadElf,
    BadAbi,
    BadPlatform,
    OutOfMemory,
    NotSupported,
    BadAddress,
    SegmentOfUndersizeAlign,
    SegmentOfZeroSize,
    SegmentOfOverlap,
    SegmentOfBadLayout,
    AlignNotSupported,
    PermissionNotSupported,
}

impl From<ParseProgramsError> for LoadError {
    fn from(_: ParseProgramsError) -> Self {
        Self::BadElf
    }
}

impl From<ParseProgramError> for LoadError {
    fn from(_: ParseProgramError) -> Self {
        Self::BadElf
    }
}

fully!(MemoryCreateError, LoadError;
    OutOfMemory => OutOfMemory,
    UndersizeAlign => SegmentOfUndersizeAlign
);
fully!(AreaMapError, LoadError;
    ZeroSize => SegmentOfZeroSize,
    OutOfRange => NotSupported,
    Overlapping => SegmentOfOverlap,
    BadAddress => BadAddress,
    AlignNotSupported => AlignNotSupported,
    PermissionNotSupported => PermissionNotSupported
);
