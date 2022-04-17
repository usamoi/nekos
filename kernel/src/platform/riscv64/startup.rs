use super::backtrace::resolve;
use super::config::CONFIG;
use super::paging::RawPaging;
use super::time::HartTime;
use super::trap::TrapFrame;
use crate::prelude::*;
use alloc::collections::BTreeMap;
use base::cell::SingletonCell;
use core::alloc::Layout;
use core::fmt::Write;
use core::mem::MaybeUninit;
use core::sync::atomic::{AtomicUsize, Ordering};
use fdt::{node::FdtNode, Fdt};
use owo_colors::OwoColorize;
use riscv::register::{satp, scause, sie, sscratch, stval, stvec};
use rt::mem::MemoryBuilder;
use rt::paging::Paging;
use rt::thread::{current, Thread};
use rt::time::Instant;

pub type GlobalBuilder = Vec<PAddr>;
pub type ThreadsBuilder = BTreeMap<usize, Thread>;

pub struct KernelBuilder {
    pub segments: BTreeMap<VAddr, (PAddr, usize, Permission)>,
}

impl KernelBuilder {
    pub fn new() -> Self {
        Self {
            segments: BTreeMap::new(),
        }
    }
    pub fn map(&mut self, vaddr: VAddr, paddr: PAddr, size: usize, permission: Permission) {
        assert_eq!(vaddr.to_usize() % 4096, 0);
        assert_eq!(paddr.to_usize() % 4096, 0);
        assert_eq!(size % 4096, 0);
        if size == 0 {
            return;
        }
        if let Some((&conflict, _)) = self.segments.range(vaddr..).next() {
            if vaddr + size > conflict {
                panic!("kernel mappings are overlapping");
            }
        }
        self.segments.insert(vaddr, (paddr, size, permission));
    }
}

extern "C" {
    static _wake: Symbol;
    static _trampoline_start: Symbol;
    static _trampoline_switch: Symbol;
    static _trampoline_trap_handler: Symbol;
    static _trampoline_end: Symbol;
    static _text_start: Symbol;
    static _text_end: Symbol;
    static _rodata_start: Symbol;
    static _rodata_end: Symbol;
    static _data_start: Symbol;
    static _data_end: Symbol;
    static _tdata_start: Symbol;
    static _tdata_end: Symbol;
    static _tbss_start: Symbol;
    static _tbss_end: Symbol;
    static _bss_start: Symbol;
    static _bss_end: Symbol;
    static _brk_start: Symbol;
    static _brk_tls_start: Symbol;
    static _brk_tls_end: Symbol;
    static _brk_stack_bot: Symbol;
    static _brk_stack_top: Symbol;
    static _brk_ptr: Symbol;
    static _kernel_address: Symbol;
}

pub fn address(symbol: &'static Symbol) -> PAddr {
    PAddr::new(0x80000000usize) + (symbol.as_vaddr() - unsafe { _kernel_address.as_vaddr() })
}

const FAULTSTACK_SIZE: usize = 8 * 1024;

#[repr(C, align(16))]
struct Stack<const N: usize>([MaybeUninit<u8>; N]);

impl<const N: usize> Stack<N> {
    const fn new() -> Self {
        Self([MaybeUninit::uninit(); N])
    }
}

#[thread_local]
static mut FAULTSTACK: Stack<FAULTSTACK_SIZE> = Stack::new();

pub struct ExtraEE {
    pub frequency: u64,
    pub trapphys: PAddr,
    pub trapaddr: VAddr,
    pub tls: usize,
    pub stack: Segment<usize>,
}

pub static EXTRA: SingletonCell<BTreeMap<usize, ExtraEE>> = SingletonCell::new();

core::arch::global_asm!(include_str!("startup.asm"));

core::arch::global_asm!(
"
    .section    .brk.stack,\"wa\",@nobits
    .globl _brk_stack_bot
    .globl _brk_stack_top
    .balign 16
_brk_stack_bot:
    .space {SIZE}
_brk_stack_top:
",
    SIZE = const config::STACK_SIZE
);

pub static mut TRAMPOLINE: VAddr = VAddr::new(0);

#[thread_local]
pub static mut ID: usize = usize::MAX;

#[no_mangle]
static SATP: AtomicUsize = AtomicUsize::new(0);

#[no_mangle]
unsafe extern "C" fn _start(cpuid: usize, opaque: *const u8) -> ! {
    let bss_size = _bss_start.size_between(&_bss_end);
    core::slice::from_raw_parts_mut(_bss_start.as_mut_ptr::<u8>(), bss_size).fill(0);
    let tls = _brk_tls_start.as_mut_ptr::<u8>();
    let tdata_size = _tdata_start.size_between(&_tdata_end);
    let tbss_size = _tbss_start.size_between(&_tbss_end);
    let master = core::slice::from_raw_parts(_tdata_start.as_ptr::<u8>(), tdata_size);
    core::slice::from_raw_parts_mut(tls, tdata_size).copy_from_slice(master);
    core::slice::from_raw_parts_mut(tls.add(tdata_size), tbss_size).fill(0);
    ID = cpuid;
    rt::log::init_global();
    info!("Hello, RISC-V.");
    rust::alloc::init_global();
    let (region_builder, mut global_builder, threads_builder) = scan(opaque);
    let memory_ptr = region_builder.ptr;
    rt::mem::init_global(region_builder);
    mem::frames::init_global();
    rust::panic::init_global(|| {
        let trapframe = EXTRA[&current().id()].trapaddr.to_usize() as *mut TrapFrame;
        (*trapframe).fault_counter += 1;
    });
    rt::thread::init_global(threads_builder);
    TRAMPOLINE = CONFIG.global().start() + global_builder.len() * 4096;
    global_builder.push(address(&_trampoline_start));
    let paging = Arc::new(RawPaging::new());
    for i in 0..256 {
        let vaddr = VAddr::new(0x40000000 * i);
        let paddr = PAddr::new(0x40000000 * i);
        paging
            .map(vaddr, paddr, 0x40000000, Permission::RW, false, false)
            .unwrap();
    }
    let mut kernel_builder = KernelBuilder::new();
    kernel_builder.map(
        _text_start.as_vaddr(),
        address(&_text_start),
        _text_start.size_between(&_text_end),
        Permission::EO,
    );
    kernel_builder.map(
        _rodata_start.as_vaddr(),
        address(&_rodata_start),
        _rodata_start.size_between(&_rodata_end),
        Permission::RO,
    );
    kernel_builder.map(
        _tdata_start.as_vaddr(),
        address(&_tdata_start),
        _tdata_start.size_between(&_tdata_end),
        Permission::RO,
    );
    kernel_builder.map(
        _data_start.as_vaddr(),
        address(&_data_start),
        _data_start.size_between(&_data_end),
        Permission::RW,
    );
    kernel_builder.map(
        _bss_start.as_vaddr(),
        address(&_bss_start),
        _bss_start.size_between(&_bss_end),
        Permission::RW,
    );
    kernel_builder.map(
        _brk_start.as_vaddr(),
        address(&_brk_start),
        memory_ptr - address(&_brk_start),
        Permission::RW,
    );
    for (vaddr, (paddr, size, permission)) in kernel_builder.segments {
        for i in 0..size / 4096 {
            paging
                .map(
                    vaddr + i * 4096,
                    paddr + i * 4096,
                    4096,
                    permission,
                    false,
                    false,
                )
                .unwrap();
        }
    }
    for (index, paddr) in global_builder.iter().copied().enumerate() {
        let vaddr = CONFIG.global().start() + index * 4096;
        paging
            .map(
                vaddr,
                paddr,
                4096,
                Permission::new(true, true, true),
                false,
                true,
            )
            .unwrap();
    }
    mem::vmm::init_global(
        paging,
        CONFIG.phys(),
        CONFIG.kernel(),
        CONFIG.heap(),
        CONFIG.global(),
        CONFIG.user(),
    );
    satp::write(mem::vmm::VMM.page_table.token());
    core::arch::riscv64::sfence_vma_all();
    core::arch::riscv64::fence_i();
    mem::heap::init_global();
    sched::scheduler::init_global();
    drivers::manager::init_global();
    SATP.store(mem::vmm::VMM.page_table.token(), Ordering::SeqCst);
    for &id in rt::thread::threads().keys() {
        if id == cpuid {
            continue;
        }
        let addr = address(&_wake).to_usize();
        let opaque = EXTRA[&id].stack.wrapping_end();
        let tls = EXTRA[&id].tls;
        *(opaque as *mut usize).offset(-1) = tls;
        super::sbi::hart_start(id, addr, opaque).unwrap();
    }
    _start3();
}

#[no_mangle]
unsafe extern "C" fn _start2(cpuid: usize) -> ! {
    ID = cpuid;
    _start3();
}

unsafe fn _start3() -> ! {
    sie::set_sext();
    sie::set_ssoft();
    sie::set_stimer();
    let trap_handler = TRAMPOLINE + (_trampoline_start.size_between(&_trampoline_trap_handler));
    stvec::write(trap_handler.to_usize(), stvec::TrapMode::Direct);
    let trapframe = EXTRA[&current().id()].trapaddr.to_usize() as *mut TrapFrame;
    sscratch::write(trapframe as usize);
    debug!("trapframe = {:#p}", trapframe);
    (*trapframe).status = 0;
    (*trapframe).fault_counter = 0;
    (*trapframe).fault_handler = _fault_handler as usize;
    core::arch::asm!("mv {}, gp", out(reg)(*trapframe).fault_gp);
    core::arch::asm!("mv {}, tp", out(reg)(*trapframe).fault_tp);
    (*trapframe).fault_sp = FAULTSTACK.0.as_mut_ptr_range().end as usize;
    (*trapframe).switch_satp = mem::vmm::VMM.page_table.token();
    rt::time::init_local(Box::new(HartTime {
        freq: EXTRA[&current().id()].frequency,
    }));
    cfg_if::cfg_if! {
        if #[cfg(test)] {
            crate::harness_main();
            panic!("harness_main exited.");
        } else {
            crate::kernel_main();
        }
    }
}

unsafe fn scan(src: *const u8) -> (MemoryBuilder, GlobalBuilder, ThreadsBuilder) {
    let mut global_builder = Vec::new();
    let dt_magic = u32::from_be(*(src as *const u32).offset(0));
    let dt_total_size = u32::from_be(*(src as *const u32).offset(1)) as usize;
    if dt_magic != 0xd00dfeed {
        panic!("no device tree is detected.")
    }
    let src_s = core::slice::from_raw_parts(src, dt_total_size);
    let dest_p = alloc::alloc::alloc(Layout::for_value(src_s));
    let dest_s = core::slice::from_raw_parts_mut(dest_p, dt_total_size);
    dest_s.copy_from_slice(src_s);
    let dt = Fdt::new(dest_s).expect("bad device tree");
    let mut region_builder;
    {
        let memory = dt.memory();
        let mut regions = memory.regions();
        let region = regions.next().unwrap();
        if regions.next().is_some() {
            panic!("do not support > 1 memory regions");
        }
        let addr = PAddr::new(region.starting_address as usize);
        let size = region.size.unwrap();
        region_builder = MemoryBuilder::new(by_size(addr, size).unwrap());
        region_builder.brk(address(&_brk_ptr));
        region_builder.alloc_buffer();
    }
    let mut threads_builder = ThreadsBuilder::new();
    let mut extras = BTreeMap::new();
    for cpu in dt.cpus() {
        let id = cpu.ids().first();
        threads_builder.insert(id, rt::thread::Thread {});
        let extra = {
            let trapphys = region_builder.alloc(MapLayout::new(4096, 4096).unwrap());
            let addr = CONFIG.global().start() + global_builder.len() * 4096;
            global_builder.push(trapphys);
            ExtraEE {
                frequency: cpu.timebase_frequency() as u64,
                trapphys,
                trapaddr: addr,
                tls: if current().id() == id {
                    let tls;
                    core::arch::asm!("mv {}, tp", out(reg) tls);
                    tls
                } else {
                    let layout =
                        Layout::from_size_align(_brk_tls_start.size_between(&_brk_tls_end), 4096)
                            .unwrap();
                    let tls = alloc::alloc::alloc(layout);
                    assert!(!tls.is_null());
                    let tdata_size = _tdata_start.size_between(&_tdata_end);
                    let tbss_size = _tbss_start.size_between(&_tbss_end);
                    let master =
                        core::slice::from_raw_parts(_tdata_start.as_ptr::<u8>(), tdata_size);
                    core::slice::from_raw_parts_mut(tls, tdata_size).copy_from_slice(master);
                    core::slice::from_raw_parts_mut(tls.add(tdata_size), tbss_size).fill(0);
                    tls as usize
                },
                stack: if current().id() == id {
                    by_points(_brk_stack_bot.as_usize(), _brk_stack_top.as_usize()).unwrap()
                } else {
                    let stack_layout = Layout::from_size_align(config::STACK_SIZE, 16).unwrap();
                    let stack_addr = alloc::alloc::alloc(stack_layout) as usize;
                    assert!(stack_addr != 0);
                    by_size(stack_addr, config::STACK_SIZE).unwrap()
                },
            }
        };
        extras.insert(id, extra);
    }
    EXTRA.initialize(extras);
    for node in dt.all_nodes() {
        solve(node);
    }
    (region_builder, global_builder, threads_builder)
}

fn solve(node: FdtNode) {
    if let Some(compatible) = node.compatible() {
        let mut all = compatible.all();
        if all.any(|s| s == "virtio,mmio") {
            let reg = node.property("reg").unwrap().value;
            assert!(reg.len() == 2 * core::mem::size_of::<usize>());
            let addr =
                usize::from_be_bytes(reg[0..core::mem::size_of::<usize>()].try_into().unwrap());
            let size =
                usize::from_be_bytes(reg[core::mem::size_of::<usize>()..].try_into().unwrap());
            let int = node
                .interrupts()
                .map(Iterator::collect)
                .unwrap_or_else(Vec::new);
            drivers::manager::register(PAddr::new(addr), size, int);
        }
    }
}

unsafe extern "C" fn _fault_handler() -> ! {
    let mut handle = rt::io::stdout().lock();
    writeln!(handle).unwrap();

    write!(handle, "{}", "Fault".red()).unwrap();
    if let Some(ms) = Instant::maybe_now()
        .map(|x| x - Instant::ZERO)
        .map(|x| x.as_millis())
    {
        write!(handle, " [{:#2}.{:#03}]", ms / 1000, ms % 1000).unwrap();
    }
    write!(handle, " [CPU {}]", current().id()).unwrap();

    let t = &mut *(EXTRA[&current().id()].trapaddr.to_usize() as *mut TrapFrame);
    writeln!(handle, "[TrapFrame]").unwrap();
    writeln!(handle, "ra = {:#x}", t.ctx.regs[1]).unwrap();
    writeln!(handle, "sp = {:#x}", t.ctx.regs[2]).unwrap();
    writeln!(handle, "gp = {:#x}", t.ctx.regs[3]).unwrap();
    writeln!(handle, "tp = {:#x}", t.ctx.regs[4]).unwrap();
    writeln!(handle, "x5 = {:#x}", t.ctx.regs[5]).unwrap();
    writeln!(handle, "x6 = {:#x}", t.ctx.regs[6]).unwrap();
    writeln!(handle, "x7 = {:#x}", t.ctx.regs[7]).unwrap();
    writeln!(handle, "fp = {:#x}", t.ctx.regs[8]).unwrap();
    writeln!(handle, "x9 = {:#x}", t.ctx.regs[9]).unwrap();
    for i in 10..32 {
        writeln!(handle, "x{} = {:#x}", i, t.ctx.regs[i]).unwrap();
    }
    for i in 0..32 {
        writeln!(handle, "f{} = {:#x}", i, t.ctx.fregs[i]).unwrap();
    }
    writeln!(handle, "sstatus = {:#x}", t.ctx.sstatus).unwrap();
    writeln!(handle, "sepc = {:#x}", t.ctx.sepc).unwrap();
    writeln!(handle, "scause = {:?}", scause::read().cause()).unwrap();
    writeln!(handle, "stval = {:#x}", stval::read()).unwrap();

    writeln!(handle, "[Backtrace]").unwrap();

    for frame in resolve(EXTRA[&current().id()].stack, t.ctx.regs[8], t.ctx.regs[2]) {
        writeln!(handle, "{:?}", frame).unwrap();
    }
    rt::process::abort();
}
