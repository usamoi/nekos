use crate::prelude::*;
use mem::frames::BoxError;

#[derive(Debug, Clone)]
pub enum MemoryCreateError {
    UndersizeAlign,
    OutOfMemory,
}

fully!(BoxError, MemoryCreateError; OutOfMemory, UndersizeAlign);
