use crate::prelude::*;
use arrayvec::ArrayVec;
use rt::backtrace::BacktraceFrame;

extern "C" {
    static _text_start: Symbol;
    static _text_end: Symbol;
}

pub unsafe fn resolve(
    stack: Segment<usize>,
    mut fp: usize,
    mut sp: usize,
) -> ArrayVec<BacktraceFrame, { config::BACKTRACE }> {
    let mut ans = ArrayVec::new();
    while fp % 16 == 0 && sp % 16 == 0 && !ans.is_full() && stack.contains(fp) && stack.contains(sp)
    {
        let ra = *(fp as *const usize).offset(-1);
        let nx = *(fp as *const usize).offset(-2);
        if ra < _text_start.as_usize() || _text_end.as_usize() <= ra {
            break;
        }
        ans.push(BacktraceFrame { ra, sp });
        sp = fp;
        fp = nx;
    }
    ans
}
