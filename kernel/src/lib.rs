#![no_std]
#![no_main]
// clippy
#![allow(clippy::identity_op)]
#![allow(clippy::missing_safety_doc)]
#![allow(clippy::new_without_default)]
#![allow(clippy::type_complexity)]
// features
#![allow(incomplete_features)]
#![feature(adt_const_params, generic_const_exprs)]
#![feature(alloc_error_handler)]
#![feature(asm_const)]
#![feature(associated_type_defaults)]
#![feature(const_convert, const_option, const_trait_impl, const_try)]
#![feature(custom_test_frameworks)]
#![feature(decl_macro)]
#![feature(extern_types)]
#![feature(inline_const)]
#![feature(int_log, int_roundings)]
#![feature(label_break_value)]
#![feature(negative_impls)]
#![feature(never_type)]
#![feature(new_uninit)]
#![feature(panic_info_message)]
#![feature(stdsimd)]
#![feature(stmt_expr_attributes)]
#![feature(thread_local)]
// test
#![reexport_test_harness_main = "harness_main"]
#![test_runner(self::rust::tests::runner)]

extern crate alloc;

pub mod arch;
pub mod common;
pub mod config;
pub mod drivers;
pub mod mem;
pub mod prelude;
pub mod proc;
pub mod rust;
pub mod sched;
pub mod user;

pub fn kernel_main() -> ! {
    sched::scheduler::forever();
}
