pub mod hsm;
pub mod legacy;
pub mod timer;

use num_enum::{IntoPrimitive, TryFromPrimitive};

#[repr(isize)]
#[derive(Debug, Clone, Copy, TryFromPrimitive, IntoPrimitive, PartialEq, Eq)]
pub enum SBIError {
    Success = 0,
    Failed = -1,
    NotSupported = -2,
    InvalidParam = -3,
    Denied = -4,
    InvalidAddress = -5,
    AlreadyAvailable = -6,
    AlreadyStarted = -7,
    AlreadyStopped = -8,
}

#[derive(Debug, Clone, Copy)]
pub struct SBIResult {
    pub err: SBIError,
    pub ret: usize,
}

impl SBIResult {
    fn new(err: isize, ret: usize) -> Option<SBIResult> {
        Some(SBIResult {
            err: SBIError::try_from(err).ok()?,
            ret,
        })
    }
    pub fn unwrap(self) -> usize {
        if !matches!(self.err, SBIError::Success) {
            panic!("called `SBIResult::unwrap()` on an not successful value");
        }
        self.ret
    }
}

macro ecall {
    ($exid: expr, $fnid: expr) => {{
        use ::core::arch::asm;

        let err: isize;
        let ret: usize;
        asm!(
            "ecall", in("x16") ($fnid), in("x17") ($exid),
            lateout("x10") err, lateout("x11") ret
        );
        SBIResult::new(err, ret).unwrap()
    }},
    ($exid: expr, $fnid: expr, $a0: expr) => {{
        use ::core::arch::asm;

        let err: isize;
        let ret: usize;
        asm!(
            "ecall", in("x16") ($fnid), in("x17") ($exid),
            in("x10") ($a0),
            lateout("x10") err, lateout("x11") ret
        );
        SBIResult::new(err, ret).unwrap()
    }},
    ($exid: expr, $fnid: expr, $a0: expr, $a1: expr) => {{
        use ::core::arch::asm;

        let err: isize;
        let ret: usize;
        asm!(
            "ecall", in("x16") ($fnid), in("x17") ($exid),
            in("x10") ($a0), in("x11") ($a1),
            lateout("x10") err, lateout("x11") ret
        );
        SBIResult::new(err, ret).unwrap()
    }},
    ($exid: expr, $fnid: expr, $a0: expr, $a1: expr, $a2: expr) => {{
        use ::core::arch::asm;

        let err: isize;
        let ret: usize;
        asm!(
            "ecall", in("x16") ($fnid), in("x17") ($exid),
            in("x10") ($a0), in("x11") ($a1), in("x12") ($a2),
            lateout("x10") err, lateout("x11") ret
        );
        SBIResult::new(err, ret).unwrap()
    }},
}
