use crate::arch::macros::thread_pointer;
use crate::prelude::*;
use arch::sbi::legacy::{remote_fence_i, remote_sfence_vma};
use arch::time::SystemTime;
use core::cell::Cell;
use crossbeam::atomic::AtomicCell;
use riscv::register::sie;
use spin::Once;

#[derive(Debug, Clone, Copy)]
pub struct ConfigStack {
    pub top: *const u8,
    pub bot: *const u8,
}

unsafe impl Sync for ConfigStack {}
unsafe impl Send for ConfigStack {}

#[derive(Debug, Clone, Copy)]
pub struct ConfigFrequency {
    pub frequency: u64,
}

pub struct Config {
    id: usize,
    stack: AtomicCell<Option<ConfigStack>>,
    frequency: AtomicCell<Option<ConfigFrequency>>,
}

impl Config {
    pub const fn new(id: usize) -> Config {
        Config {
            id,
            stack: AtomicCell::new(None),
            frequency: AtomicCell::new(None),
        }
    }
    pub const fn id(&self) -> usize {
        self.id
    }
    pub fn set_stack(&self, stack: ConfigStack) {
        let ans = self.stack.swap(Some(stack));
        assert!(ans.is_none(), "the value can be set only once");
    }
    pub fn get_stack(&self) -> Option<ConfigStack> {
        self.stack.load()
    }
    pub fn stack(&self) -> ConfigStack {
        self.get_stack().expect("the value remains uninitialized")
    }
    pub fn set_frequency(&self, frequency: ConfigFrequency) {
        let ans = self.frequency.swap(Some(frequency));
        assert!(ans.is_none(), "the value can be set only once");
    }
    pub fn get_frequency(&self) -> Option<ConfigFrequency> {
        self.frequency.load()
    }
    pub fn frequency(&self) -> ConfigFrequency {
        self.get_frequency()
            .expect("the value remains uninitialized")
    }
}

pub struct Local {
    id: Cell<Option<usize>>,
}

impl Local {
    const fn new() -> Self {
        Self {
            id: Cell::new(None),
        }
    }
    pub fn set_id(&self, id: usize) {
        assert!(self.id.get().is_none(), "the value can be set only once");
        self.id.set(Some(id));
    }
    pub fn get_id(&self) -> Option<usize> {
        if arch::macros::thread_pointer!() != 0 {
            self.id.get()
        } else {
            None
        }
    }
    pub fn id(&self) -> usize {
        self.get_id().unwrap()
    }
    pub fn get_config(&self) -> Option<&Config> {
        CONFIGS.get_config(self.get_id()?)
    }
    pub fn config(&self) -> &Config {
        self.get_config().unwrap()
    }
    pub fn local_set_timer(&self, time: SystemTime) {
        arch::sbi::timer::set_timer(time.into_raw()).unwrap();
    }
    pub fn local_fence_ins(&self) {
        unsafe {
            core::arch::riscv64::fence_i();
        }
    }
    pub fn local_fence_tlb(&self) {
        unsafe {
            core::arch::riscv64::sfence_vma_all();
        }
    }
}

#[thread_local]
static LOCAL: Local = Local::new();

pub fn checked_local() -> Option<ThreadLocalRef<Local>> {
    if thread_pointer!() != 0 {
        Some(unsafe { ThreadLocalRef::new(&LOCAL) })
    } else {
        None
    }
}

pub fn local() -> ThreadLocalRef<Local> {
    checked_local().unwrap()
}

#[derive(Debug, Clone, Copy)]
pub struct Remote(usize);

impl Remote {
    pub const fn all() -> Self {
        Remote(usize::MAX)
    }
    pub fn remote_fence_ins(&self) {
        remote_fence_i(&self.0)
    }
    pub fn remote_fence_tlb(&self, start: usize, size: usize) {
        remote_sfence_vma(&self.0, start, size)
    }
}

pub struct Configs {
    config: [Once<Config>; 64],
}

impl Configs {
    pub const fn new() -> Self {
        Self {
            config: [const { Once::new() }; 64],
        }
    }
    pub fn set_config(&self, id: usize, cpu: Config) {
        self.config[id].call_once(|| cpu);
    }
    pub fn get_config(&self, id: usize) -> Option<&Config> {
        self.config[id].get()
    }
    pub fn config_iter(&self) -> impl Iterator<Item = &Config> {
        self.config.iter().filter_map(|x| x.get())
    }
    pub fn config_len(&self) -> usize {
        self.config_iter().count()
    }
    pub fn config(&self, id: usize) -> &Config {
        self.get_config(id).unwrap()
    }
}

pub static CONFIGS: Configs = Configs::new();

pub unsafe fn init_start(cpuid: usize) {
    arch::cpu::LOCAL.set_id(cpuid);
    sie::set_sext();
    sie::set_stimer();
}
