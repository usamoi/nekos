use crate::prelude::*;
use rt::trap::Trapping;

core::arch::global_asm!(include_str!("trap.asm"));

#[repr(C)]
#[derive(Debug, Clone)]
pub struct RawTrapping {
    pub regs: [usize; 32],
    pub fregs: [usize; 32],
    pub sstatus: usize,
    pub sepc: usize,
}

impl Trapping for RawTrapping {
    fn new(privilege: Privilege, pc: VAddr, sp: VAddr, tp: VAddr, opaque: usize) -> RawTrapping {
        use Privilege::*;
        let mut this = RawTrapping {
            regs: [0; 32],
            fregs: [0; 32],
            sstatus: 0,
            sepc: 0,
        };
        this.sstatus = match privilege {
            User => 0x8000000000006000,
            Kernel => todo!(),
        };
        this.sepc = pc.to_usize();
        this.regs[2] = sp.to_usize();
        this.regs[4] = tp.to_usize();
        this.regs[10] = opaque;
        this
    }
    fn solve_breakpoint(&mut self) {
        self.sepc += 2;
    }
    fn solve_syscall(&mut self, x: Option<usize>, y: Option<usize>) {
        if let Some(value) = x {
            self.regs[10] = value;
        }
        if let Some(value) = y {
            self.regs[11] = value;
        }
        self.sepc += 4;
    }
}

#[repr(C, align(4096))]
pub struct TrapFrame {
    pub ctx: RawTrapping,     // 0
    pub status: usize,        // 66
    pub fault_counter: usize, // 67
    pub fault_handler: usize, // 68
    pub fault_gp: usize,      // 69
    pub fault_tp: usize,      // 70
    pub fault_sp: usize,      // 71
    pub switch_sp: usize,     // 72
    pub switch_satp: usize,   // 73
}
