use super::{Context, TRAMPOLINE, TRAPFRAME};
use crate::prelude::*;
use arch::paging::PageTableToken;
use mem::vmm::pt;
use riscv::register::{scause, stval};

extern "C" {
    static _trampoline_start: LinkerSymbol;
    static _trampoline_switch: LinkerSymbol;
}

#[repr(C)]
#[derive(Debug, Clone)]
pub struct Switch {
    ctx: Context,
    satp: PageTableToken,
}

impl Switch {
    pub const fn new(pt: PageTableToken, pc: VAddr, sp: VAddr, tp: VAddr) -> Switch {
        Switch {
            ctx: {
                let mut ctx = Context::new();
                ctx.sstatus = 0x8000000000006000;
                ctx.sepc = pc.to_usize();
                ctx.regs[2] = sp.to_usize();
                ctx.regs[4] = tp.to_usize();
                ctx
            },
            satp: pt,
        }
    }
    pub fn set_opaque(&mut self, opaque: usize) {
        self.ctx.regs[10] = opaque;
    }
    pub fn set_pc(&mut self, pc: VAddr) {
        self.ctx.sepc = pc.to_usize();
    }
    pub fn solve_breakpoint(&mut self) {
        self.ctx.sepc += 2;
    }
    pub fn solve_syscall(&mut self, ret: Result<usize, Errno>) {
        match ret {
            Ok(value) => {
                self.ctx.regs[10] = 0;
                self.ctx.regs[11] = value;
            }
            Err(errno) => {
                self.ctx.regs[10] = errno.into_raw().get() as usize;
            }
        }
        // it's valid because only `ecall` raise a syscall,
        // and the length of ecall instruction is 4 bytes
        self.ctx.sepc += 4;
    }
    pub unsafe fn switch(&mut self) -> Trap {
        let switch = *TRAMPOLINE + (_trampoline_switch.as_vaddr() - _trampoline_start.as_vaddr());
        (*TRAPFRAME.get()).ctx = self.ctx.clone();
        core::mem::transmute::<VAddr, extern "C" fn(PageTableToken)>(switch)(self.satp);
        self.ctx = (*TRAPFRAME.get()).ctx.clone();
        make_trap(scause::read().bits(), stval::read(), &self.ctx)
    }
}

const fn make_trap(scause: usize, stval: usize, ctx: &Context) -> Trap {
    use self::Exception::*;
    use self::Interrupt::*;
    use self::Trap::*;
    const EX: usize = 0;
    const IN: usize = 1usize << (usize::BITS - 1);
    match scause {
        // exception
        x if x == EX | 2 => Exception(IllegalInstruction),
        x if x == EX | 0 => Exception(Misaligned {
            op: MemoryOperation::Instruction,
        }),
        x if x == EX | 6 => Exception(Misaligned {
            op: MemoryOperation::Store,
        }),
        x if x == EX | 12 => Exception(PageFault {
            op: MemoryOperation::Instruction,
            addr: VAddr::new(stval),
        }),
        x if x == EX | 13 => Exception(PageFault {
            op: MemoryOperation::Load,
            addr: VAddr::new(stval),
        }),
        x if x == EX | 15 => Exception(PageFault {
            op: MemoryOperation::Store,
            addr: VAddr::new(stval),
        }),
        x if x == EX | 8 => Exception(Syscall {
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
        x if x == EX | 3 => Exception(Breakpoint),
        x if x == IN | 1 => Interrupt(Software),
        x if x == IN | 5 => Interrupt(Timer),
        x if x == IN | 9 => Interrupt(Hardware),
        _ => Unknown,
    }
}

pub unsafe fn init_start() {
    (*TRAPFRAME.get()).switch_satp = pt().into_raw();
}
