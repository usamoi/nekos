use crate::prelude::*;
use crossbeam::atomic::AtomicCell;
use owo_colors::OwoColorize;
use rt::thread::threads;
use rt::time::Instant;

pub fn runner(tests: &[&dyn Fn()]) -> ! {
    static LOCK: AtomicCell<usize> = AtomicCell::new(0);
    if threads().len() != 1 + LOCK.fetch_add(1) {
        loop {
            core::hint::spin_loop();
        }
    }
    println!("running {} tests", tests.len());
    let start_time = Instant::now();
    let mut count = 0;
    for &test in tests {
        count += 1;
        print!("test {}", count);
        print!(" ...");
        test();
        print!("{}", "passed".green());
        println!();
    }
    let duration = Instant::now().duration_since(start_time);
    println!("test result: done. {} passed.", count);
    println!("finished in {:?}.", duration);
    rt::process::abort();
}
