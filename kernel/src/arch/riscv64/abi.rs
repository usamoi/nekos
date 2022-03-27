use crate::prelude::*;
use arrayvec::ArrayVec;
use common::backtrace::BacktraceFrame;

pub macro stack_pointer() {
    #[allow(unused_unsafe)]
    unsafe {
        let reg: usize;
        core::arch::asm!("mv {}, sp", out(reg) reg);
        reg
    }
}

pub macro thread_pointer() {
    #[allow(unused_unsafe)]
    unsafe {
        let reg: usize;
        core::arch::asm!("mv {}, tp", out(reg) reg);
        reg
    }
}

pub macro frame_pointer() {
    #[allow(unused_unsafe)]
    unsafe {
        let reg: usize;
        core::arch::asm!("mv {}, fp", out(reg) reg);
        reg
    }
}

pub const STACK_ALIGN: usize = 16;
pub const STACK_OFFSET: usize = 0;

pub const ELF_EABI: u16 = 243;

pub unsafe fn set_thread_pointer(tp: usize) {
    core::arch::asm!("mv tp, {}", in(reg) tp);
}

pub unsafe fn get_backtrace(
    stack: Segment<usize>,
    text: Segment<usize>,
    mut fp: usize,
    mut sp: usize,
) -> ArrayVec<BacktraceFrame, { config::BACKTRACE_LIMIT }> {
    let mut ans = ArrayVec::new();
    while fp % 16 == 0 && sp % 16 == 0 && !ans.is_full() && stack.contains(fp) && stack.contains(sp)
    {
        let ra = *(fp as *const usize).offset(-1);
        let next_fp = *(fp as *const usize).offset(-2);
        if !text.contains(ra) {
            break;
        }
        ans.push(BacktraceFrame { ra, fp, sp });
        sp = fp;
        fp = next_fp;
    }
    ans
}
