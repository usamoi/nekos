#![no_std]
#![feature(panic_info_message)]
#![feature(linkage)]
#![feature(format_args_nl)]
#![feature(decl_macro)]
#![feature(never_type)]

use syscalls::thread_exit;

pub mod lang;
pub mod macros;
pub mod syscalls;

#[no_mangle]
extern "C" fn _start() {
    main();
    thread_exit();
}

#[no_mangle]
#[linkage = "weak"]
fn main() {}
