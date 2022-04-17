use crate::prelude::*;
use arrayvec::ArrayVec;
use rt::backtrace::BacktraceFrame;
use rt::paging::Paging;
use rt::trap::Trapping;

pub trait Platform {
    // types
    type Trapping: Trapping;
    type Paging: Paging;
    // constants
    const ABI_STACK_ALIGN: usize;
    const ABI_STACK_OFFSET: usize;
    const ABI_ELF_ABI: u16;
    const PAGING_PERMISSION: [bool; 8];
    const PAGING_ALIGN: [bool; 64];
    // functions
    fn id() -> usize;
    fn abort() -> !;
    fn write(s: &str);
    unsafe fn backtrace() -> ArrayVec<BacktraceFrame, { config::BACKTRACE }>;
    unsafe fn trap_switch(ctx: &mut Self::Trapping, pt: &Self::Paging) -> Trap;
}

pub struct P;

impl P {
    pub fn check_align(align: usize) -> bool {
        assert!(align.is_power_of_two());
        P::PAGING_ALIGN[align.log2() as usize]
    }
    pub fn check_permission(permission: Permission) -> bool {
        P::PAGING_PERMISSION[permission.as_u8() as usize]
    }
}
