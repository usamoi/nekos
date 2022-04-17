use super::backtrace::resolve;
use super::paging::RawPaging;
use super::startup::{ID, TRAMPOLINE};
use super::trap::{RawTrapping, TrapFrame};
use crate::platform::riscv64::startup::EXTRA;
use crate::prelude::*;
use arrayvec::ArrayVec;
use riscv::register::{scause, stval};
use rt::backtrace::BacktraceFrame;
use rt::platform::P;
use rt::thread::current;

extern "C" {
    static _trampoline_start: Symbol;
    static _trampoline_switch: Symbol;
}

impl Platform for P {
    type Trapping = RawTrapping;
    type Paging = RawPaging;
    const ABI_STACK_ALIGN: usize = 16;
    const ABI_STACK_OFFSET: usize = 0;
    const ABI_ELF_ABI: u16 = 243;
    const PAGING_PERMISSION: [bool; 8] = {
        let mut ans = [false; 8];
        ans[Permission::EO.as_u8() as usize] = true;
        ans[Permission::RO.as_u8() as usize] = true;
        ans[Permission::RW.as_u8() as usize] = true;
        ans[Permission {
            read: true,
            write: false,
            execute: true,
        }
        .as_u8() as usize] = true;
        ans[Permission {
            read: true,
            write: true,
            execute: true,
        }
        .as_u8() as usize] = true;
        ans
    };
    const PAGING_ALIGN: [bool; 64] = {
        let mut ans = [false; 64];
        ans[12] = true;
        ans[21] = true;
        ans[30] = true;
        ans
    };

    fn id() -> usize {
        unsafe { ID }
    }

    fn abort() -> ! {
        super::sbi::shutdown();
        unreachable!()
    }

    fn write(s: &str) {
        for c in s.bytes() {
            super::sbi::console_putchar(c.into());
        }
    }

    #[inline(always)]
    unsafe fn backtrace() -> ArrayVec<BacktraceFrame, { config::BACKTRACE }> {
        use crate::config::BACKTRACE;
        use crate::rt::backtrace::BacktraceFrame;
        use crate::rt::thread::current;
        use ::arrayvec::ArrayVec;

        let fp: usize;
        let sp: usize;
        core::arch::asm!("mv {}, fp", out(reg) fp);
        core::arch::asm!("mv {}, sp", out(reg) sp);
        let stack = EXTRA[&current().id()].stack;
        resolve(stack, fp, sp) as ArrayVec<BacktraceFrame, BACKTRACE>
    }

    unsafe fn trap_switch(ctx: &mut RawTrapping, pt: &RawPaging) -> Trap {
        use Exception::*;
        use Interrupt::*;
        use Trap::*;
        assert_ne!(TRAMPOLINE, VAddr::new(0));
        let switch = TRAMPOLINE + (_trampoline_switch.as_vaddr() - _trampoline_start.as_vaddr());
        let trapframe = EXTRA[&current().id()].trapaddr.to_usize() as *mut TrapFrame;
        (*trapframe).ctx = ctx.clone();
        core::mem::transmute::<VAddr, extern "C" fn(usize)>(switch)(pt.token());
        *ctx = (*trapframe).ctx.clone();
        let stval = stval::read();
        match scause::read().bits() {
            // exception
            0x0 => TrapException(Misaligned {
                access: Access::Instruction,
                addr: VAddr::new(stval),
            }),
            0x2 => TrapException(IllegalInstruction),
            0x3 => TrapException(Breakpoint),
            0x4 => TrapException(Misaligned {
                access: Access::Load,
                addr: VAddr::new(stval),
            }),
            0x6 => TrapException(Misaligned {
                access: Access::Store,
                addr: VAddr::new(stval),
            }),
            0x8 => TrapException(Syscall {
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
            0xc => TrapException(PageFault {
                access: Access::Instruction,
                addr: VAddr::new(stval),
            }),
            0xd => TrapException(PageFault {
                access: Access::Load,
                addr: VAddr::new(stval),
            }),
            0xf => TrapException(PageFault {
                access: Access::Store,
                addr: VAddr::new(stval),
            }),
            0x8000000000000001 => TrapInterrupt(Software { value: stval }),
            0x8000000000000005 => TrapInterrupt(Timer),
            0x8000000000000009 => TrapInterrupt(Hardware { value: stval }),
            _ => TrapUnknown,
        }
    }
}
