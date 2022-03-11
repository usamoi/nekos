use crate::prelude::*;
use proc::loader::LoadError;
use proc::thread::ThreadCreateError;

#[derive(Debug)]
pub enum ProcessCreateError {
    LoadError,
    OutOfMemory,
    OutOfVirtualMemory,
}

impl From<LoadError> for ProcessCreateError {
    fn from(_e: LoadError) -> Self {
        use ProcessCreateError::*;
        LoadError
    }
}

partially!(ProcessSpawnError, ProcessCreateError; OutOfMemory, OutOfVirtualMemory);

#[derive(Debug)]
pub enum ProcessSpawnError {
    BadStatus,
    OutOfMemory,
    OutOfVirtualMemory,
}

fully!(ThreadCreateError, ProcessSpawnError; OutOfMemory, OutOfVirtualMemory);

#[derive(Debug)]
pub enum ProcessStopError {
    BadStatus,
}
