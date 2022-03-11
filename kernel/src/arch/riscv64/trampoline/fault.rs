use super::TRAPFRAME;
use crate::prelude::*;
use arch::backtrace::*;
use arch::cpu::SystemTime;
use arch::cpu::LOCAL;
use arch::power::POWER;
use arch::stdout::STDOUT;
use core::alloc::Layout;
use core::fmt::Write;
use crossbeam::atomic::AtomicCell;
use owo_colors::OwoColorize;
use riscv::register::{scause, stval};

extern "C" {
    static _text_start: LinkerSymbol;
    static _text_end: LinkerSymbol;
    #[link_name = "__global_pointer$"]
    static _global_pointer: LinkerSymbol;
}

pub unsafe fn panic_handler() {
    if LOCAL.get_id().is_some() {
        (*TRAPFRAME.get()).fault_counter += 1;
    }
}

pub unsafe fn init_start() {
    (*TRAPFRAME.get()).fault_counter = 0;
    (*TRAPFRAME.get()).fault_handler = fault_handler as usize;
    (*TRAPFRAME.get()).fault_gp = _global_pointer.as_usize();
    (*TRAPFRAME.get()).fault_tp = arch::macros::thread_pointer!();
    (*TRAPFRAME.get()).fault_sp = {
        let stack_layout =
            Layout::from_size_align(config::FAULT_STACK_SIZE, arch::consts::ABI_STACK_ALIGN)
                .unwrap();
        let stack_bot = alloc::alloc::alloc(stack_layout);
        stack_bot.add(config::FAULT_STACK_SIZE) as usize
    };
}

unsafe extern "C" fn fault_handler() -> ! {
    #[link_section = ".data"]
    static LOCK: AtomicCell<u8> = AtomicCell::new(0);

    while LOCK.compare_exchange(0, 1).is_err() {
        core::hint::spin_loop();
    }

    let s = &mut *STDOUT.write.lock();
    writeln!(s).unwrap();

    write!(s, "{}", "Fault".red()).unwrap();
    if let Some(ms) = SystemTime::now().checked_duration_since(SystemTime::ZERO).map(|x| x.as_millis()) {
        write!(s, " [{:#2}.{:#03}]", ms / 1000, ms % 1000).unwrap();
    }
    if let Some(id) = LOCAL.get_id() {
        write!(s, " [CPU {}]", id).unwrap();
    }

    let t = &mut *super::TRAPFRAME.get();
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

    let local_stack = LOCAL.config().stack();
    let stack = by_points(local_stack.bot as usize, local_stack.top as usize).unwrap();
    let text = by_points(_text_start.as_usize(), _text_end.as_usize()).unwrap();

    for frame in resolve(stack, text, t.ctx.regs[8], t.ctx.regs[2]) {
        writeln!(s, "{:?}", frame).unwrap();
    }

    POWER.shutdown();
}
