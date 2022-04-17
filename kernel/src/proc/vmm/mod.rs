mod area;
pub use self::area::*;

mod map;
pub use self::map::*;

mod space;
pub use self::space::*;

use crate::prelude::*;
use mem::pages::*;

#[derive(Debug)]
pub enum AreaCreateError {
    ZeroSize,
    OutOfRange,
    Overlapping,
}

#[derive(Debug)]
pub enum AreaFindCreateError {
    ZeroSize,
    OutOfRange,
    OutOfVirtualMemory,
}

#[derive(Debug)]
pub enum AreaMapError {
    ZeroSize,
    OutOfRange,
    Overlapping,
    BadAddress,
    AlignNotSupported,
    PermissionNotSupported,
}

#[derive(Debug)]
pub enum AreaFindMapError {
    ZeroSize,
    OutOfVirtualMemory,
    AlignNotSupported,
    PermissionNotSupported,
}

#[derive(Debug)]
pub enum AreaUnmapError {
    UnmapAnArea,
    NotFound,
}

#[derive(Debug)]
pub enum AreaReadError {
    OutOfRange,
    BadRead,
    PermissionDenied,
}

#[derive(Debug)]
pub enum AreaWriteError {
    OutOfRange,
    BadWrite,
    PermissionDenied,
}

fully!(PagesNewError, AreaCreateError; ZeroSize);
fully!(PagesNewError, AreaFindCreateError; ZeroSize);
fully!(PagesFindError, AreaFindCreateError; ZeroSize, OutOfVirtualMemory);
fully!(PagesAcquireError, AreaCreateError; ZeroSize, OutOfRange, Overlapping);
partially!(PagesAcquireError, AreaFindCreateError; ZeroSize, OutOfRange);
fully!(PagesAcquireError, AreaMapError; ZeroSize, OutOfRange, Overlapping);
fully!(PagesFindError, AreaFindMapError; ZeroSize, OutOfVirtualMemory);
fully!(PagesReleaseError, AreaUnmapError; NotFound);
partially!(PagesAcquireError, AreaFindMapError; ZeroSize);
