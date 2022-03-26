use crate::prelude::*;
use arch::cpu::CONFIGS;
use arch::power::POWER;
use arch::time::MachineInstant;
use crossbeam::atomic::AtomicCell;
use owo_colors::OwoColorize;

pub fn runner(tests: &[&dyn Fn()]) -> ! {
    static LOCK: AtomicCell<usize> = AtomicCell::new(0);
    if CONFIGS.config_len() != 1 + LOCK.fetch_add(1) {
        loop {
            core::hint::spin_loop();
        }
    }
    println!("running {} tests", tests.len());
    let start_time = MachineInstant::now();
    let mut count = 0;
    for test in tests {
        count += 1;
        print!("test {}", count);
        print!(" ...");
        test();
        print!("{}", "passed".green());
        println!();
    }
    let duration = MachineInstant::now().duration_since(start_time);
    println!("test result: done. {} passed.", count);
    println!("finished in {:?}.", duration);
    POWER.shutdown();
}
