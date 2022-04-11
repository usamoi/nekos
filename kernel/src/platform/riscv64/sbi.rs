#![allow(dead_code)]

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

macro legacy_ecall {
    ($id: expr) => {{
        let ret: usize;
        core::arch::asm!("ecall", in("x17") ($id),
            lateout("x10") ret);
        ret
    }},
    ($id: expr, $a0: expr) => {{
        let ret: usize;
        core::arch::asm!("ecall", in("x17") ($id),
            in("x10") ($a0),
            lateout("x10") ret);
        ret
    }},
    ($id: expr, $a0: expr, $a1: expr) => {{
        let ret: usize;
        core::arch::asm!("ecall", in("x17") ($id),
            in("x10") ($a0), in("x11") ($a1),
            lateout("x10") ret);
        ret
    }},
    ($id: expr, $a0: expr, $a1: expr, $a2: expr) => {{
        let ret: usize;
        core::arch::asm!("ecall", in("x17") ($id),
            in("x10") ($a0), in("x11") ($a1), in("x12") ($a2),
            lateout("x10") ret);
        ret
    }},
    ($id: expr, $a0: expr, $a1: expr, $a2: expr, $a3: expr) => {{
        let ret: usize;
        core::arch::asm!("ecall", in("x17") ($id),
            in("x10") ($a0), in("x11") ($a1), in("x12") ($a2), in("x13") ($a3),
            lateout("x10") ret);
        ret
    }},
}

const CONSOLE_PUTCHAR: usize = 1;
const CLEAR_IPI: usize = 3;
const SEND_IPI: usize = 4;
const REMOTE_FENCE_I: usize = 5;
const REMOTE_SFENCE_VMA: usize = 6;
const REMOTE_SFENCE_VMA_ASID: usize = 7;
const SHUTDOWN: usize = 8;

pub fn console_putchar(ch: i32) {
    unsafe {
        legacy_ecall!(CONSOLE_PUTCHAR, ch);
    }
}

pub fn clear_ipi() {
    unsafe {
        legacy_ecall!(CLEAR_IPI);
    }
}

pub fn send_ipi(hart_mask: *const usize) {
    unsafe {
        legacy_ecall!(SEND_IPI, hart_mask);
    }
}

pub fn remote_fence_i(hart_mask: *const usize) {
    unsafe {
        legacy_ecall!(REMOTE_FENCE_I, hart_mask);
    }
}

pub fn remote_sfence_vma(hart_mask: *const usize, start: usize, size: usize) {
    unsafe {
        legacy_ecall!(REMOTE_SFENCE_VMA, hart_mask, start, size);
    }
}

pub fn remote_sfence_vma_asid(hart_mask: *const usize, start: usize, size: usize, asid: usize) {
    unsafe {
        legacy_ecall!(REMOTE_SFENCE_VMA_ASID, hart_mask, start, size, asid);
    }
}

pub fn shutdown() {
    unsafe {
        legacy_ecall!(SHUTDOWN);
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

const HSM_ID: usize = 0x48534D;
const HSM_HART_START: usize = 0;
const HSM_HART_STOP: usize = 1;
const HSM_HART_GET_STATUS: usize = 2;

pub fn hart_start(id: usize, addr: usize, opaque: usize) -> SBIResult {
    unsafe { ecall!(HSM_ID, HSM_HART_START, id, addr, opaque) }
}

pub fn hart_stop() -> SBIResult {
    unsafe { ecall!(HSM_ID, HSM_HART_STOP) }
}

pub fn hart_get_status(id: usize) -> SBIResult {
    unsafe { ecall!(HSM_ID, HSM_HART_GET_STATUS, id) }
}

const TIMER_ID: usize = 0x54494D45;
const TIMER_SET_TIMER: usize = 0;

pub fn timer_set_timer(time: u64) -> SBIResult {
    unsafe { ecall!(TIMER_ID, TIMER_SET_TIMER, time) }
}
