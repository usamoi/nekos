#![no_std]
#![no_main]
#![feature(format_args_nl)]
#![feature(thread_local)]

#[macro_use]
extern crate nekos_initproc;

use core::cell::Cell;
use nekos_initproc::syscalls::thread_create;

#[thread_local]
static X: Cell<usize> = Cell::new(0);

extern "C" fn sub(opaque: usize) -> ! {
    println!("[{}]", opaque as u8 as char);
    while X.get() < 100 {
        print!("{}", X.get());
        X.set(X.get() + 1);
    }
    loop {}
}

#[no_mangle]
fn main() {
    for i in 1..10 {
        thread_create(sub, (b'A' + i) as usize).unwrap();
    }
}
