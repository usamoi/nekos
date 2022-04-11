use super::backtrace::backtrace_resolve;
use super::paging::RawPaging;
use super::paging::RawPagingGroup;
use super::trap::{RawTrapContext, TRAMPOLINE, TRAPFRAME};
use crate::prelude::*;
use arrayvec::ArrayVec;
use core::time::Duration;
use riscv::register::satp;
use riscv::register::{scause, stval, time};
use rt::backtrace::BacktraceFrame;
use rt::paging::Paging;
use rt::paging::PagingToken;
use rt::platform::P;
use rt::thread::current;
use rt::thread::maybe_threads;

extern "C" {
    static _trampoline_start: LinkerSymbol;
    static _trampoline_switch: LinkerSymbol;
}

#[thread_local]
pub static mut ID: usize = usize::MAX;

impl Platform for P {
    type TrapContext = RawTrapContext;
    type Paging = RawPaging;
    type PagingGroup = RawPagingGroup;
    const STACK_ALIGN: usize = 16;
    const STACK_OFFSET: usize = 0;
    const ELF_EABI: u16 = 243;
    const PAGING_PHYS: Segment<VAddr> = by_points(
        VAddr::new(0x0000000000000000),
        VAddr::new(0x0000004000000000),
    )
    .unwrap();
    const PAGING_KERNEL: Segment<VAddr> = by_points(
        VAddr::new(0xFFFFFFC000000000),
        VAddr::new(0xFFFFFFC040000000),
    )
    .unwrap();
    const PAGING_HEAP: Segment<VAddr> = by_points(
        VAddr::new(0xFFFFFFC040000000),
        VAddr::new(0xFFFFFFFFC0000000),
    )
    .unwrap();
    const PAGING_GLOBAL: Segment<VAddr> =
        Segment::new(VAddr::new(0xFFFFFFFFC0000000), None).unwrap();
    fn thread_id() -> usize {
        unsafe { ID }
    }

    fn thread_flush_ins() {
        unsafe {
            core::arch::riscv64::fence_i();
        }
    }

    fn thread_flush_tlb() {
        unsafe {
            core::arch::riscv64::sfence_vma_all();
        }
    }

    fn process_abort() -> ! {
        super::sbi::shutdown();
        unreachable!()
    }

    fn io_write(s: &str) {
        for c in s.bytes() {
            super::sbi::console_putchar(c.into());
        }
    }

    fn time_now() -> u64 {
        time::read64()
    }

    fn time_timer(time: u64) {
        super::sbi::timer_set_timer(time).unwrap();
    }

    fn time_sub_maybe(start: u64, end: u64) -> Option<Duration> {
        let freq = maybe_threads()?[&current().id()].extra.frequency;
        Some(Duration::from_micros((end - start) * 1_000_000 / freq))
    }

    fn time_add_maybe(ins: u64, dur: Duration) -> Option<u64> {
        let freq = maybe_threads()?[&current().id()].extra.frequency;
        Some(ins + dur.as_micros() as u64 * (freq / 1_000_000))
    }

    fn paging_align(align: usize) -> bool {
        assert!(align.is_power_of_two());
        match align.log2() {
            12 => true,
            21 => true,
            30 => true,
            _ => false,
        }
    }

    fn paging_permission(permission: Permission) -> bool {
        match permission {
            Permission::RO => true,
            Permission::RW => true,
            Permission::EO => true,
            Permission {
                read: true,
                write: false,
                execute: true,
            } => true,
            _ => false,
        }
    }

    #[inline(always)]
    unsafe fn backtrace() -> ArrayVec<BacktraceFrame, { config::BACKTRACE_LIMIT }> {
        extern "C" {
            static _text_start: LinkerSymbol;
            static _text_end: LinkerSymbol;
        }
        use crate::config::BACKTRACE_LIMIT;
        use crate::mem::defines::by_points;
        use crate::mem::defines::LinkerSymbol;
        use crate::rt::backtrace::BacktraceFrame;
        use crate::rt::thread::current;
        use crate::rt::thread::threads;
        use ::arrayvec::ArrayVec;

        let fp: usize;
        let sp: usize;
        core::arch::asm!("mv {}, fp", out(reg) fp);
        core::arch::asm!("mv {}, sp", out(reg) sp);
        let stack = threads()[&current().id()].stack;
        let text = by_points(_text_start.as_usize(), _text_end.as_usize()).unwrap();
        backtrace_resolve(stack, text, fp, sp) as ArrayVec<BacktraceFrame, BACKTRACE_LIMIT>
    }

    unsafe fn trap_switch(ctx: &mut RawTrapContext, pt: PagingToken) -> Trap {
        use self::Exception::*;
        use self::Interrupt::*;
        use self::Trap::*;
        let switch = *TRAMPOLINE + (_trampoline_switch.as_vaddr() - _trampoline_start.as_vaddr());
        (*TRAPFRAME.get()).ctx = ctx.clone();
        core::mem::transmute::<VAddr, extern "C" fn(PagingToken)>(switch)(pt);
        *ctx = (*TRAPFRAME.get()).ctx.clone();
        let stval = stval::read();
        match scause::read().bits() {
            // exception
            0x0 => Exception(Misaligned {
                access: Access::Instruction,
                addr: VAddr::new(stval),
            }),
            0x2 => Exception(IllegalInstruction),
            0x3 => Exception(Breakpoint),
            0x4 => Exception(Misaligned {
                access: Access::Load,
                addr: VAddr::new(stval),
            }),
            0x6 => Exception(Misaligned {
                access: Access::Store,
                addr: VAddr::new(stval),
            }),
            0x8 => Exception(Syscall {
                id: ctx.regs[17],
                args: [
                    ctx.regs[10],
                    ctx.regs[11],
                    ctx.regs[12],
                    ctx.regs[13],
                    ctx.regs[14],
                    ctx.regs[15],
                ],
            }),
            0xc => Exception(PageFault {
                access: Access::Instruction,
                addr: VAddr::new(stval),
            }),
            0xd => Exception(PageFault {
                access: Access::Load,
                addr: VAddr::new(stval),
            }),
            0xf => Exception(PageFault {
                access: Access::Store,
                addr: VAddr::new(stval),
            }),
            0x8000000000000001 => Interrupt(Software { value: stval }),
            0x8000000000000005 => Interrupt(Timer),
            0x8000000000000009 => Interrupt(Hardware { value: stval }),
            _ => Unknown,
        }
    }

    unsafe fn paging_phys(page_table: &RawPaging) {
        for i in 0..256 {
            let vaddr = VAddr::new(0x40000000 * i);
            let paddr = PAddr::new(0x40000000 * i);
            page_table
                .map(vaddr, paddr, 0x40000000, Permission::RW, false, false)
                .unwrap();
        }
    }

    unsafe fn paging_token_switch(pt: PagingToken) {
        satp::write(pt.0);
        core::arch::riscv64::sfence_vma_all();
        core::arch::riscv64::fence_i();
    }
}
