use crate::prelude::*;
use core::fmt::Debug;

#[derive(Debug, Clone)]
pub struct User {
    ctx: <P as Platform>::TrapContext,
    pt: rt::paging::PagingToken,
}

impl User {
    pub fn new(pt: rt::paging::PagingToken, pc: VAddr, sp: VAddr, tp: VAddr) -> User {
        User {
            ctx: {
                let mut ctx = <P as Platform>::TrapContext::new();
                ctx.set_user();
                ctx.set_pc(pc.to_usize());
                ctx.set_sp(sp.to_usize());
                ctx.set_tp(tp.to_usize());
                ctx
            },
            pt,
        }
    }
    pub fn set_opaque(&mut self, opaque: usize) {
        self.ctx.set_opaque(opaque);
    }
    pub fn set_pc(&mut self, pc: VAddr) {
        self.ctx.set_pc(pc.to_usize());
    }
    pub fn solve_breakpoint(&mut self) {
        self.ctx.solve_breakpoint();
    }
    pub fn solve_syscall(&mut self, ret: Result<usize, Errno>) {
        self.ctx.solve_syscall(ret);
    }
    pub unsafe fn switch(&mut self) -> Trap {
        P::trap_switch(&mut self.ctx, self.pt)
    }
}

pub trait TrapContext: Debug + Copy + Send + Sync {
    fn new() -> Self;
    fn set_user(&mut self);
    fn set_pc(&mut self, pc: usize);
    fn set_sp(&mut self, sp: usize);
    fn set_tp(&mut self, tp: usize);
    fn set_opaque(&mut self, opaque: usize);
    fn solve_breakpoint(&mut self);
    fn solve_syscall(&mut self, ret: Result<usize, Errno>);
}
