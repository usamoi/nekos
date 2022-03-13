use crate::prelude::*;
use arch::cpu::ConfigStack;
use core::alloc::Layout;
use core::sync::atomic::AtomicUsize;
use core::sync::atomic::Ordering;

extern "C" {
    static _bss_start: LinkerSymbol;
    static _bss_end: LinkerSymbol;
    static _stack_bot: LinkerSymbol;
    static _stack_top: LinkerSymbol;
    static _startup2: LinkerSymbol;
}

cfg_if::cfg_if! {
    if #[cfg(riscv64_paging = "sv39")] {
        core::arch::global_asm!(include_str!("sv39.asm"));
    } else if #[cfg(riscv64_paging = "sv48")] {
        core::arch::global_asm!(include_str!("sv48.asm"));
    } else {
        compile_error!("unknown paging");
    }
}

core::arch::global_asm!(include_str!("startup.asm"));

core::arch::global_asm!(
"
    .section .uninit
    .globl _stack_top
    .balign 16
_stack_bot:
    .space {SIZE}
_stack_top:
",
    SIZE = const config::STACK_SIZE
);

#[inline(always)]
unsafe fn memset_bss() {
    let bss_size = _bss_end.as_vaddr() - _bss_start.as_vaddr();
    core::slice::from_raw_parts_mut(_bss_start.as_mut_ptr::<u8>(), bss_size).fill(0);
}

#[no_mangle]
static mut SATP: AtomicUsize = AtomicUsize::new(0);

#[no_mangle]
unsafe extern "C" fn _start(cpuid: usize, opaque: *const u8) -> ! {
    memset_bss();
    common::logging::init_boot();
    info!("Hello, RISC-V.");
    info!("/* A cat employee is wanted here */");
    info!("BOOT CPUID: {}", cpuid);
    info!("booting");
    mem::heap::HEAP.init_fallback();
    arch::hardware::init_boot(opaque);
    mem::frames::init_boot();
    mem::vmm::init_boot();
    arch::trampoline::init_boot();
    sched::scheduler::init_boot();
    mem::heap::HEAP.init_slab();
    SATP.store(mem::vmm::pt().into_raw(), Ordering::SeqCst);
    for config in arch::cpu::CONFIGS.config_iter() {
        if config.id() as usize != cpuid {
            let stack_layout = Layout::from_size_align(config::STACK_SIZE, 16).unwrap();
            let stack_bot = alloc::alloc::alloc(stack_layout);
            assert!(!stack_bot.is_null());
            let stack_top = stack_bot.add(config::STACK_SIZE);
            config.set_stack(ConfigStack {
                bot: stack_bot,
                top: stack_top,
            });
            let addr = _startup2.as_paddr();
            arch::sbi::hsm::hart_start(config.id(), addr.to_usize(), stack_top as usize).unwrap();
        } else {
            config.set_stack(ConfigStack {
                bot: _stack_bot.as_ptr(),
                top: _stack_top.as_ptr(),
            });
        }
    }
    _start2(cpuid);
}

#[no_mangle]
unsafe extern "C" fn _start2(cpuid: usize) -> ! {
    info!("starting");
    arch::tls::init_start();
    arch::cpu::init_start(cpuid);
    arch::trampoline::init_start();
    arch::trampoline::fault::init_start();
    arch::trampoline::switch::init_start();
    _main();
}

unsafe fn _main() -> ! {
    info!("ready");
    cfg_if::cfg_if! {
        if #[cfg(test)] {
            crate::harness_main();
            panic!("harness_main exited.");
        } else {
            crate::kernel_main();
        }
    }
}
