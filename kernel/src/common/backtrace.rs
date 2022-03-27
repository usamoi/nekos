use core::fmt::Debug;

pub struct BacktraceFrame {
    pub ra: usize,
    pub fp: usize,
    pub sp: usize,
}

impl Debug for BacktraceFrame {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "ra = {:#x}, fp = {:#x}, sp = {:#x}",
            self.ra, self.fp, self.sp
        )
    }
}

pub macro backtrace() {{
    extern "C" {
        static _text_start: LinkerSymbol;
        static _text_end: LinkerSymbol;
    }

    use ::arrayvec::ArrayVec;
    use $crate::arch::abi::frame_pointer;
    use $crate::arch::abi::get_backtrace;
    use $crate::arch::abi::stack_pointer;
    use $crate::arch::cpu::local;
    use $crate::common::backtrace::BacktraceFrame;
    use $crate::config::BACKTRACE_LIMIT;
    use $crate::mem::defines::by_points;
    use $crate::mem::defines::LinkerSymbol;

    unsafe {
        let fp = frame_pointer!();
        let sp = stack_pointer!();
        let local_stack = local().config().stack();
        let stack = by_points(local_stack.bot as usize, local_stack.top as usize).unwrap();
        let text = by_points(_text_start.as_usize(), _text_end.as_usize()).unwrap();
        get_backtrace(stack, text, fp, sp) as ArrayVec<BacktraceFrame, BACKTRACE_LIMIT>
    }
}}
