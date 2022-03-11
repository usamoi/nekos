use crate::prelude::*;
use mem::pages::*;

#[derive(Debug)]
pub enum KAreaCreateError {
    ZeroSize,
    OutOfRange,
    Overlapping,
}

fully!(PagesNewError, KAreaCreateError; ZeroSize);
fully!(PagesAcquireError, KAreaCreateError; ZeroSize, OutOfRange, Overlapping);

#[derive(Debug)]
pub enum KAreaFindCreateError {
    ZeroSize,
    OutOfRange,
    OutOfVirtualMemory,
}

fully!(PagesNewError, KAreaFindCreateError; ZeroSize);
fully!(PagesFindError, KAreaFindCreateError; ZeroSize, OutOfVirtualMemory);
partially!(PagesAcquireError, KAreaFindCreateError; ZeroSize, OutOfRange);

#[derive(Debug)]
pub enum KAreaMapError {
    ZeroSize,
    OutOfRange,
    Overlapping,
    BadAddress,
    AlignNotSupported,
    PermissionNotSupported,
}

fully!(PagesAcquireError, KAreaMapError; ZeroSize, OutOfRange, Overlapping);

#[derive(Debug)]
pub enum KAreaFindMapError {
    ZeroSize,
    OutOfVirtualMemory,
    AlignNotSupported,
    PermissionNotSupported,
}

fully!(PagesFindError, KAreaFindMapError; ZeroSize, OutOfVirtualMemory);
partially!(PagesAcquireError, KAreaFindMapError; ZeroSize);

#[derive(Debug)]
pub enum KAreaUnmapError {
    UnmapAnArea,
    NotFound,
}

fully!(PagesReleaseError, KAreaUnmapError; NotFound);
