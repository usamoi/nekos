use super::buddy::BuddyError;
use crate::prelude::*;

#[derive(Debug)]
pub enum PagesNewError {
    ZeroSize,
}

#[derive(Debug)]
pub enum PagesAcquireError {
    ZeroSize,
    OutOfRange,
    Overlapping,
}

#[derive(Debug)]
pub enum PagesReleaseError {
    NotFound,
}

#[derive(Debug)]
pub enum PagesFindError {
    ZeroSize,
    OutOfVirtualMemory,
}

fully!(BuddyError, PagesFindError; ZeroSize => ZeroSize, OutOfBounds => OutOfVirtualMemory);
