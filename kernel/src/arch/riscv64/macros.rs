pub macro stack_pointer() {
    #[allow(unused_unsafe)]
    unsafe {
        let reg: usize;
        core::arch::asm!("mv {}, sp", out(reg) reg);
        reg
    }
}

pub macro thread_pointer() {
    #[allow(unused_unsafe)]
    unsafe {
        let reg: usize;
        core::arch::asm!("mv {}, tp", out(reg) reg);
        reg
    }
}

pub macro frame_pointer() {
    #[allow(unused_unsafe)]
    unsafe {
        let reg: usize;
        core::arch::asm!("mv {}, fp", out(reg) reg);
        reg
    }
}
