#![no_std]
#![no_main]
// clippy
#![allow(clippy::identity_op)]
#![allow(clippy::missing_safety_doc)]
#![allow(clippy::new_without_default)]
#![allow(clippy::type_complexity)]
// features
#![feature(allocator_api)]
#![feature(asm_const)]
#![feature(associated_type_defaults)]
#![feature(bool_to_option)]
#![feature(const_btree_new)]
#![feature(const_convert)]
#![feature(const_option)]
#![feature(const_trait_impl)]
#![feature(const_try)]
#![feature(custom_test_frameworks)]
#![feature(decl_macro)]
#![feature(extern_types)]
#![feature(int_log)]
#![feature(int_roundings)]
#![feature(lang_items)]
#![feature(layout_for_ptr)]
#![feature(map_first_last)]
#![feature(negative_impls)]
#![feature(never_type)]
#![feature(new_uninit)]
#![feature(once_cell)]
#![feature(panic_info_message)]
#![feature(ptr_metadata)]
#![feature(slice_ptr_get)]
#![feature(stdsimd)]
#![feature(thread_local)]
// test
#![reexport_test_harness_main = "harness_main"]
#![test_runner(self::rust::tests::runner)]

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
