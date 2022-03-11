use crate::prelude::*;

pub struct Power {}

impl Power {
    const fn new() -> Power {
        Power {}
    }
    pub fn shutdown(&self) -> ! {
        arch::sbi::legacy::shutdown();
        unreachable!()
    }
}

pub static POWER: Power = Power::new();
