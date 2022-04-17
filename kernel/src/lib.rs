#![no_std]
#![no_main]
// clippy
#![allow(clippy::identity_op)]
#![allow(clippy::missing_safety_doc)]
#![allow(clippy::new_without_default)]
#![allow(clippy::type_complexity)]
// features
#![feature(allocator_api)]
#![feature(arbitrary_self_types)]
#![feature(asm_const)]
#![feature(associated_type_defaults)]
#![feature(const_btree_new)]
#![feature(const_option)]
#![feature(custom_test_frameworks)]
#![feature(decl_macro)]
#![feature(int_log)]
#![feature(int_roundings)]
#![feature(lang_items)]
#![feature(map_first_last)]
#![feature(negative_impls)]
#![feature(never_type)]
#![feature(panic_info_message)]
#![feature(ptr_metadata)]
#![feature(stdsimd)]
#![feature(thread_local)]
#![feature(try_trait_v2)]
// test
#![reexport_test_harness_main = "harness_main"]
#![test_runner(crate::rust::tests::runner)]

extern crate alloc;

pub mod base;
pub mod config;
pub mod drivers;
pub mod fs;
pub mod mem;
mod platform;
pub mod prelude;
pub mod proc;
pub mod rt;
pub mod rust;
pub mod sched;
pub mod user;

pub fn kernel_main() -> ! {
    sched::scheduler::forever();
}
