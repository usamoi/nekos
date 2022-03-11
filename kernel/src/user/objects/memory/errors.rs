use crate::prelude::*;
use mem::frames::BoxError;

#[derive(Debug, Clone)]
pub enum MemoryCreateError {
    ZeroSize,
    UndersizeAlign,
    OutOfMemory,
}

fully!(BoxError, MemoryCreateError; OutOfMemory, UndersizeAlign, ZeroSize);
