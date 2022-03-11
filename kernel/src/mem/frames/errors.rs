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
    ZeroSize,
    UndersizeAlign,
    OutOfMemory,
}

fully!(BuddyError, FramesAllocError; ZeroSize => ZeroSize, OutOfBounds => OutOfMemory);

#[derive(Debug)]
pub enum FramesDeallocError {
    ZeroSize,
}

#[derive(Debug)]
pub enum BoxError {
    ZeroSize,
    UndersizeAlign,
    OutOfMemory,
}

fully!(FramesAllocError, BoxError; ZeroSize, UndersizeAlign, OutOfMemory);
