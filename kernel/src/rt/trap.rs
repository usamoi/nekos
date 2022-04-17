use crate::prelude::*;
use core::fmt::Debug;

#[derive(Debug)]
pub enum Exception {
    IllegalInstruction,
    Misaligned { access: Access, addr: VAddr },
    PageFault { access: Access, addr: VAddr },
    Syscall { id: usize, args: Arguments },
    Breakpoint,
}

#[derive(Debug)]
pub enum Interrupt {
    Timer,
    Software { value: usize },
    Hardware { value: usize },
}

#[derive(Debug)]
pub enum Trap {
    TrapUnknown,
    TrapException(Exception),
    TrapInterrupt(Interrupt),
}

pub enum Privilege {
    User,
    Kernel,
}

pub trait Trapping: Debug + Clone + Send + Sync {
    fn new(privilege: Privilege, pc: VAddr, sp: VAddr, tp: VAddr, opaque: usize) -> Self;
    fn solve_breakpoint(&mut self);
    fn solve_syscall(&mut self, x: Option<usize>, y: Option<usize>);
}
