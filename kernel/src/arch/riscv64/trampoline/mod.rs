pub mod fault;
pub mod switch;

use crate::prelude::*;
use common::basic::Singleton;
use mem::frames::PhysBox;
use mem::vmm::{KMapUnsafe, SPACE};
use riscv::register::{sscratch, stvec};

extern "C" {
    static _trampoline_start: LinkerSymbol;
    static _trampoline_end: LinkerSymbol;
    static _trampoline_trap_handler: LinkerSymbol;
}

core::arch::global_asm!(include_str!("trampoline.asm"));

#[repr(C)]
#[derive(Debug, Clone)]
struct Context {
    pub regs: [usize; 32],
    pub fregs: [usize; 32],
    pub sstatus: usize,
    pub sepc: usize,
}

impl Context {
    const fn new() -> Context {
        Context {
            regs: [0; 32],
            fregs: [0; 32],
            sstatus: 0,
            sepc: 0,
        }
    }
}

#[repr(C, align(4096))]
struct TrapFrame {
    pub ctx: Context,         // 0
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
            ctx: Context::new(),
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

static TRAMPOLINE: Singleton<VAddr> = Singleton::new();

#[thread_local]
static TRAPFRAME: Singleton<PhysBox<TrapFrame>> = Singleton::new();

pub unsafe fn init_global() {
    TRAMPOLINE.init({
        let trappoline_paddr = _trampoline_start.as_paddr();
        let trampoline_size = _trampoline_end.as_vaddr() - _trampoline_start.as_vaddr();
        let trampoline_layout = MapLayout::new(trampoline_size, 4096).unwrap();
        let trampoline_map =
            Arc::new(KMapUnsafe::new(trappoline_paddr, trampoline_layout).unwrap());
        SPACE
            .global
            .root
            .find_map(trampoline_map, MapPermission::EO, true)
            .unwrap()
    });
}

pub unsafe fn init_local() {
    stvec::write(
        {
            let trampoline_trap_handler =
                *TRAMPOLINE + (_trampoline_trap_handler.as_vaddr() - _trampoline_start.as_vaddr());
            trampoline_trap_handler.to_usize()
        },
        stvec::TrapMode::Direct,
    );
    TRAPFRAME.init(PhysBox::new(TrapFrame::new()).unwrap());
    sscratch::write({
        let trapframe_paddr = TRAPFRAME.paddr();
        let trapframe_layout =
            MapLayout::from_alloc_layout(core::alloc::Layout::new::<TrapFrame>());
        let trapframe_map = Arc::new(KMapUnsafe::new(trapframe_paddr, trapframe_layout).unwrap());
        let trapframe_vaddr = SPACE
            .global
            .root
            .find_map(trapframe_map, MapPermission::RW, true);
        trapframe_vaddr.unwrap().to_usize()
    });
}
