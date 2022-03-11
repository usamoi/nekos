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

impl From<AreaReadError> for SideEffect {
    fn from(e: AreaReadError) -> Self {
        use AreaReadError as E;
        use ProcessFault::*;
        match e {
            E::OutOfRange => SideEffect::KillProcess(ProcessDeath::Fault(Segment {
                op: MemoryOperation::Load,
            })),
            E::BadRead => SideEffect::KillProcess(ProcessDeath::Fault(Segment {
                op: MemoryOperation::Load,
            })),
            E::PermissionDenied => SideEffect::KillProcess(ProcessDeath::Fault(Segment {
                op: MemoryOperation::Load,
            })),
        }
    }
}

impl From<AreaWriteError> for SideEffect {
    fn from(e: AreaWriteError) -> Self {
        use AreaWriteError as E;
        use ProcessFault::*;
        match e {
            E::OutOfRange => SideEffect::KillProcess(ProcessDeath::Fault(Segment {
                op: MemoryOperation::Store,
            })),
            E::BadWrite => SideEffect::KillProcess(ProcessDeath::Fault(Segment {
                op: MemoryOperation::Store,
            })),
            E::PermissionDenied => SideEffect::KillProcess(ProcessDeath::Fault(Segment {
                op: MemoryOperation::Store,
            })),
        }
    }
}
