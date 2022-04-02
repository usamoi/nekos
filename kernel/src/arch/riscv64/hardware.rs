use crate::{drivers::manager, prelude::*};
use arch::cpu::{Config, CONFIGS};
use core::alloc::Layout;
use fdt::{node::FdtNode, Fdt};

extern "C" {
    static _bump_start: LinkerSymbol;
}

pub unsafe fn init_global(src_p: *const u8) {
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
    for cpu in dt.cpus() {
        let cpuid = cpu.ids().first();
        let ans = Config::new(cpuid);
        ans.set_frequency(arch::cpu::ConfigFrequency {
            frequency: cpu.timebase_frequency() as u64,
        });
        CONFIGS.set_config(cpuid, ans);
    }
    {
        let memory = dt.memory();
        let mut regions = memory.regions();
        let region = regions.next().unwrap();
        if regions.next().is_some() {
            panic!("do not support > 1 memory regions");
        }
        arch::memory::CONFIG.set_start(PAddr::new(region.starting_address as usize));
        arch::memory::CONFIG.set_size(region.size.unwrap());
        arch::memory::CONFIG.set_bump(_bump_start.as_paddr());
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
