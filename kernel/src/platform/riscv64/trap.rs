use super::backtrace::backtrace_resolve;
use crate::prelude::*;
use crate::rt::trap::TrapContext;
use base::cell::{LocalSingletonCell, SingletonCell};
use core::alloc::Layout;
use core::fmt::Write;
use crossbeam::atomic::AtomicCell;
use mem::frames::FramesBox;
use mem::vmm::pt;
use mem::vmm::{KMap, SPACE};
use owo_colors::OwoColorize;
use riscv::register::{scause, sscratch, stval, stvec};
use rt::thread::current;
use rt::thread::threads;
use rt::time::Instant;

extern "C" {
    static _text_start: LinkerSymbol;
    static _text_end: LinkerSymbol;
    static _trampoline_start: LinkerSymbol;
    static _trampoline_end: LinkerSymbol;
    static _trampoline_trap_handler: LinkerSymbol;
    static _trampoline_switch: LinkerSymbol;
}

core::arch::global_asm!(include_str!("trap.asm"));

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct RawTrapContext {
    pub regs: [usize; 32],
    pub fregs: [usize; 32],
    pub sstatus: usize,
    pub sepc: usize,
}

impl TrapContext for RawTrapContext {
    fn new() -> RawTrapContext {
        RawTrapContext {
            regs: [0; 32],
            fregs: [0; 32],
            sstatus: 0,
            sepc: 0,
        }
    }
    fn set_user(&mut self) {
        self.sstatus = 0x8000000000006000;
    }
    fn set_pc(&mut self, pc: usize) {
        self.sepc = pc;
    }
    fn set_sp(&mut self, sp: usize) {
        self.regs[2] = sp;
    }
    fn set_tp(&mut self, tp: usize) {
        self.regs[4] = tp;
    }
    fn set_opaque(&mut self, opaque: usize) {
        self.regs[10] = opaque;
    }
    fn solve_breakpoint(&mut self) {
        self.sepc += 2;
    }
    fn solve_syscall(&mut self, ret: Result<usize, Errno>) {
        match ret {
            Ok(value) => {
                self.regs[10] = 0;
                self.regs[11] = value;
            }
            Err(errno) => {
                self.regs[10] = errno.into_raw() as usize;
            }
        }
        self.sepc += 4;
    }
}

#[repr(C, align(4096))]
pub struct TrapFrame {
    pub ctx: RawTrapContext,  // 0
    pub status: usize,        // 66
    pub fault_counter: usize, // 67
    pub fault_handler: usize, // 68
    pub fault_gp: usize,      // 69
    pub fault_tp: usize,      // 70
    pub fault_sp: usize,      // 71
    pub switch_sp: usize,     // 72
    pub switch_satp: usize,   // 73
}

impl TrapFrame {
    const fn new() -> TrapFrame {
        TrapFrame {
            ctx: RawTrapContext {
                regs: [0; 32],
                fregs: [0; 32],
                sstatus: 0,
                sepc: 0,
            },
            status: 0,
            fault_counter: 0,
            fault_handler: 0,
            fault_gp: 0,
            fault_tp: 0,
            fault_sp: 0,
            switch_satp: 0,
            switch_sp: 0,
        }
    }
}

pub static TRAMPOLINE: SingletonCell<VAddr> = SingletonCell::new();

#[thread_local]
pub static TRAPFRAME: LocalSingletonCell<FramesBox<TrapFrame>> = LocalSingletonCell::new();

pub unsafe fn init_global() {
    rust::panic::hook_set_hook(|| {
        if let Some(f) = TRAPFRAME.maybe() {
            (*f.get()).fault_counter += 1;
        }
    });
    TRAMPOLINE.initialize({
        let paddr = _trampoline_start.as_paddr();
        let size = _trampoline_end.as_vaddr() - _trampoline_start.as_vaddr();
        let layout = MapLayout::new(size, 4096).unwrap();
        let map = Arc::new(KMap::new(paddr, layout).unwrap());
        SPACE.global_map(map, Permission::EO)
    });
}

pub unsafe fn init_local() {
    stvec::write(
        (*TRAMPOLINE + (_trampoline_trap_handler.as_vaddr() - _trampoline_start.as_vaddr()))
            .to_usize(),
        stvec::TrapMode::Direct,
    );
    TRAPFRAME.initialize(FramesBox::new(TrapFrame::new()).unwrap());
    sscratch::write({
        let paddr = TRAPFRAME.paddr();
        let layout = MapLayout::from_rust(core::alloc::Layout::new::<TrapFrame>());
        let map = Arc::new(KMap::new(paddr, layout).unwrap());
        SPACE.global_map(map, Permission::RW).to_usize()
    });
    (*TRAPFRAME.get()).fault_counter = 0;
    (*TRAPFRAME.get()).fault_handler = fault_handler as usize;
    core::arch::asm!("mv {}, gp", out(reg)(*TRAPFRAME.get()).fault_gp);
    core::arch::asm!("mv {}, tp", out(reg)(*TRAPFRAME.get()).fault_tp);
    (*TRAPFRAME.get()).fault_sp = {
        let layout = Layout::from_size_align(config::FAULT_STACK_SIZE, 16).unwrap();
        let stack = alloc::alloc::alloc(layout) as usize;
        assert!(stack != 0);
        stack + config::FAULT_STACK_SIZE
    };
    (*TRAPFRAME.get()).switch_satp = pt().0;
}

unsafe extern "C" fn fault_handler() -> ! {
    #[link_section = ".data"]
    static LOCK: AtomicCell<u8> = AtomicCell::new(0);

    while LOCK.compare_exchange(0, 1).is_err() {
        core::hint::spin_loop();
    }

    let mut s = rt::io::stdout().lock();
    writeln!(s).unwrap();

    write!(s, "{}", "Fault".red()).unwrap();
    if let Some(ms) = Instant::now()
        .maybe_duration_since(Instant::ZERO)
        .map(|x| x.as_millis())
    {
        write!(s, " [{:#2}.{:#03}]", ms / 1000, ms % 1000).unwrap();
    }
    write!(s, " [CPU {}]", current().id()).unwrap();

    let t = &mut *TRAPFRAME.get();
    writeln!(s, "[TrapFrame]").unwrap();
    writeln!(s, "ra = {:#x}", t.ctx.regs[1]).unwrap();
    writeln!(s, "sp = {:#x}", t.ctx.regs[2]).unwrap();
    writeln!(s, "gp = {:#x}", t.ctx.regs[3]).unwrap();
    writeln!(s, "tp = {:#x}", t.ctx.regs[4]).unwrap();
    writeln!(s, "x5 = {:#x}", t.ctx.regs[5]).unwrap();
    writeln!(s, "x6 = {:#x}", t.ctx.regs[6]).unwrap();
    writeln!(s, "x7 = {:#x}", t.ctx.regs[7]).unwrap();
    writeln!(s, "fp = {:#x}", t.ctx.regs[8]).unwrap();
    writeln!(s, "x9 = {:#x}", t.ctx.regs[9]).unwrap();
    for i in 10..32 {
        writeln!(s, "x{} = {:#x}", i, t.ctx.regs[i]).unwrap();
    }
    for i in 0..32 {
        writeln!(s, "f{} = {:#x}", i, t.ctx.fregs[i]).unwrap();
    }
    writeln!(s, "sstatus = {:#x}", t.ctx.sstatus).unwrap();
    writeln!(s, "sepc = {:#x}", t.ctx.sepc).unwrap();
    writeln!(s, "scause = {:?}", scause::read().cause()).unwrap();
    writeln!(s, "stval = {:#x}", stval::read()).unwrap();

    writeln!(s, "[Backtrace]").unwrap();

    let stack = threads()[&current().id()].stack;
    let text = by_points(_text_start.as_usize(), _text_end.as_usize()).unwrap();

    for frame in backtrace_resolve(stack, text, t.ctx.regs[8], t.ctx.regs[2]) {
        writeln!(s, "{:?}", frame).unwrap();
    }
    rt::process::abort();
}
