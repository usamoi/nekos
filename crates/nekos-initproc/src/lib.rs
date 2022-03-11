#![no_std]
#![feature(panic_info_message)]
#![feature(linkage)]
#![feature(format_args_nl)]
#![feature(decl_macro)]
#![feature(never_type)]

pub mod lang;
pub mod macros;
pub mod syscalls;

#[no_mangle]
extern "C" fn _start() {
    main();
    loop {}
}

#[no_mangle]
#[linkage = "weak"]
fn main() {}
