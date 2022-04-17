use crate::prelude::*;
use arrayvec::ArrayVec;
use core::fmt::Debug;

pub struct BacktraceFrame {
    pub ra: usize,
    pub sp: usize,
}

impl Debug for BacktraceFrame {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "ra = {:#x}, sp = {:#x}", self.ra, self.sp)
    }
}

#[inline(always)]
pub unsafe fn backtrace() -> ArrayVec<BacktraceFrame, { config::BACKTRACE }> {
    P::backtrace()
}
