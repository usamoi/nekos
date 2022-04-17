#![no_std]
#![no_main]
#![feature(format_args_nl)]
#![feature(thread_local)]

#[macro_use]
extern crate nekos_initproc;

fn sub() {
    let mut x = 0;
    while x < 100 {
        print!("{}", 0);
        x += 1;
    }
}

#[no_mangle]
fn main() {
    for _ in 0..10 {
        sub();
    }
    loop {}
}
