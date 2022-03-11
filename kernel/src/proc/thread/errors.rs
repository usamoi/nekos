use crate::prelude::*;
use proc::vmm::AreaFindMapError;
use user::objects::memory::MemoryCreateError;

#[derive(Debug)]
pub enum ThreadCreateError {
    OutOfMemory,
    OutOfVirtualMemory,
}

partially!(MemoryCreateError, ThreadCreateError; OutOfMemory);
partially!(AreaFindMapError, ThreadCreateError; OutOfVirtualMemory);
