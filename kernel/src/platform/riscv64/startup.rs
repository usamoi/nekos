use super::rt::ID;
use crate::prelude::*;
use alloc::collections::BTreeMap;
use core::alloc::Layout;
use core::sync::atomic::AtomicUsize;
use core::sync::atomic::Ordering;
use drivers::manager;
use fdt::node::FdtNode;
use fdt::Fdt;
use riscv::register::sie;
use rt::mem::hook_set_memory_region;
use rt::mem::RegionBuilder;
use rt::thread::hook_set_threads;
use rt::thread::Extra;

extern "C" {
    static _startup2: LinkerSymbol;
    static _tdata_start: LinkerSymbol;
    static _tdata_end: LinkerSymbol;
    static _tbss_start: LinkerSymbol;
    static _tbss_end: LinkerSymbol;
    static _bss_start: LinkerSymbol;
    static _bss_end: LinkerSymbol;
    static _utls_start: LinkerSymbol;
    static _utls_end: LinkerSymbol;
    static _ustack_bot: LinkerSymbol;
    static _ustack_top: LinkerSymbol;
    static _guard_start: LinkerSymbol;
}

core::arch::global_asm!(include_str!("startup.asm"));

core::arch::global_asm!(
"
    .section    .uninit.ustack,\"wa\",@nobits
    .globl _ustack_bot
    .globl _ustack_top
    .balign 16
_ustack_bot:
    .space {SIZE}
_ustack_top:
",
    SIZE = const config::STACK_SIZE
);

#[no_mangle]
static mut SATP: AtomicUsize = AtomicUsize::new(0);

#[no_mangle]
unsafe extern "C" fn _start(cpuid: usize, opaque: *const u8) -> ! {
    let bss_size = _bss_start.size_between(&_bss_end);
    core::slice::from_raw_parts_mut(_bss_start.as_mut_ptr::<u8>(), bss_size).fill(0);
    let tls = _utls_start.as_mut_ptr::<u8>();
    let tdata_size = _tdata_start.size_between(&_tdata_end);
    let tbss_size = _tbss_start.size_between(&_tbss_end);
    let master = core::slice::from_raw_parts(_tdata_start.as_ptr::<u8>(), tdata_size);
    core::slice::from_raw_parts_mut(tls, tdata_size).copy_from_slice(master);
    core::slice::from_raw_parts_mut(tls.add(tdata_size), tbss_size).fill(0);
    core::arch::asm!("mv tp, {}", in(reg) tls);
    ID = cpuid;
    rt::log::init_global();
    info!("Hello, RISC-V.");
    rust::alloc::init_global();
    scan(cpuid, opaque);
    mem::frames::init_global();
    mem::vmm::init_global();
    mem::heap::init_global();
    super::trap::init_global();
    sched::scheduler::init_global();
    drivers::manager::init_global();
    SATP.store(mem::vmm::pt().0, Ordering::SeqCst);
    for (&id, thread) in rt::thread::threads() {
        if id == cpuid {
            continue;
        }
        let addr = _startup2.as_paddr().to_usize();
        let opaque = thread.stack.wrapping_end();
        super::sbi::hart_start(id, addr, opaque).unwrap();
    }
    _start3();
}

#[no_mangle]
unsafe extern "C" fn _start2(cpuid: usize) -> ! {
    let layout = Layout::from_size_align(_utls_start.size_between(&_utls_end), 4096).unwrap();
    let tls = alloc::alloc::alloc(layout);
    assert!(!tls.is_null());
    let tdata_size = _tdata_start.size_between(&_tdata_end);
    let tbss_size = _tbss_start.size_between(&_tbss_end);
    let master = core::slice::from_raw_parts(_tdata_start.as_ptr::<u8>(), tdata_size);
    core::slice::from_raw_parts_mut(tls, tdata_size).copy_from_slice(master);
    core::slice::from_raw_parts_mut(tls.add(tdata_size), tbss_size).fill(0);
    core::arch::asm!("mv tp, {}", in(reg) tls);
    ID = cpuid;
    _start3();
}

#[no_mangle]
unsafe extern "C" fn _start3() -> ! {
    sie::set_sext();
    sie::set_ssoft();
    sie::set_stimer();
    super::trap::init_local();
    cfg_if::cfg_if! {
        if #[cfg(test)] {
            crate::harness_main();
            panic!("harness_main exited.");
        } else {
            crate::kernel_main();
        }
    }
}

pub unsafe fn scan(xid: usize, src_p: *const u8) {
    let dt_magic = u32::from_be(*(src_p as *const u32).offset(0));
    let dt_total_size = u32::from_be(*(src_p as *const u32).offset(1)) as usize;
    if dt_magic != 0xd00dfeed {
        panic!("no device tree is detected.")
    }
    let src_s = core::slice::from_raw_parts(src_p, dt_total_size);
    let dest_p = alloc::alloc::alloc(Layout::for_value(src_s));
    let dest_s = core::slice::from_raw_parts_mut(dest_p, dt_total_size);
    dest_s.copy_from_slice(src_s);
    let dt = Fdt::new(dest_s).expect("bad device tree");
    let mut threads = BTreeMap::new();
    for cpu in dt.cpus() {
        let cpuid = cpu.ids().first();
        threads.insert(
            cpuid,
            rt::thread::Thread {
                stack: if xid == cpuid {
                    by_points(_ustack_bot.as_usize(), _ustack_top.as_usize()).unwrap()
                } else {
                    let stack_layout = Layout::from_size_align(config::STACK_SIZE, 16).unwrap();
                    let stack_addr = alloc::alloc::alloc(stack_layout) as usize;
                    assert!(stack_addr != 0);
                    by_size(stack_addr, config::STACK_SIZE).unwrap()
                },
                extra: Extra {
                    frequency: cpu.timebase_frequency() as u64,
                },
            },
        );
    }
    hook_set_threads(threads);
    {
        let memory = dt.memory();
        let mut regions = memory.regions();
        let region = regions.next().unwrap();
        if regions.next().is_some() {
            panic!("do not support > 1 memory regions");
        }
        let segment = by_size(
            PAddr::new(region.starting_address as usize),
            region.size.unwrap(),
        )
        .unwrap();
        let mut builder = RegionBuilder::new(segment);
        builder.alloc_addr(_guard_start.as_paddr());
        let buffer_size = (segment.end().unwrap() - segment.start()) / 4096 * 2;
        let use_buffer = builder.alloc_size(buffer_size);
        builder.set_buffer(use_buffer);
        builder.alloc_addr(builder.ptr.to_usize().next_multiple_of(4096).into());
        hook_set_memory_region(builder.finish().unwrap());
    }
    for node in dt.all_nodes() {
        solve(node);
    }
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
            manager::register(PAddr::new(addr), size, int);
        }
    }
}
