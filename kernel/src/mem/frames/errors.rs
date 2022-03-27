use super::buddy::BuddyError;
use crate::prelude::*;

#[derive(Debug)]
pub enum FramesNewError {
    ZeroSize,
    OutOfBuffer,
}

fully!(BuddyError, FramesNewError; ZeroSize => ZeroSize, OutOfBounds => OutOfBuffer);

#[derive(Debug)]
pub enum FramesAllocError {
    UndersizeAlign,
    OutOfMemory,
}

partially!(BuddyError, FramesAllocError; OutOfBounds => OutOfMemory);

#[derive(Debug)]
pub enum BoxError {
    UndersizeAlign,
    OutOfMemory,
}

fully!(FramesAllocError, BoxError; UndersizeAlign, OutOfMemory);
