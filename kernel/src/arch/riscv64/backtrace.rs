use crate::prelude::*;
use arch::common::backtrace::BacktraceFrame;
use arrayvec::ArrayVec;

pub unsafe fn resolve(
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
