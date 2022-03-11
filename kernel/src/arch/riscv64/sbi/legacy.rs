const CONSOLE_PUTCHAR: usize = 1;
const CLEAR_IPI: usize = 3;
const SEND_IPI: usize = 4;
const REMOTE_FENCE_I: usize = 5;
const REMOTE_SFENCE_VMA: usize = 6;
const REMOTE_SFENCE_VMA_ASID: usize = 7;
const SHUTDOWN: usize = 8;

macro ecall {
    ($id: expr) => {{
        let ret: usize;
        core::arch::asm!("ecall", in("x17") ($id),
            lateout("x10") ret);
        ret
    }},
    ($id: expr, $a0: expr) => {{
        let ret: usize;
        core::arch::asm!("ecall", in("x17") ($id),
            in("x10") ($a0),
            lateout("x10") ret);
        ret
    }},
    ($id: expr, $a0: expr, $a1: expr) => {{
        let ret: usize;
        core::arch::asm!("ecall", in("x17") ($id),
            in("x10") ($a0), in("x11") ($a1),
            lateout("x10") ret);
        ret
    }},
    ($id: expr, $a0: expr, $a1: expr, $a2: expr) => {{
        let ret: usize;
        core::arch::asm!("ecall", in("x17") ($id),
            in("x10") ($a0), in("x11") ($a1), in("x12") ($a2),
            lateout("x10") ret);
        ret
    }},
    ($id: expr, $a0: expr, $a1: expr, $a2: expr, $a3: expr) => {{
        let ret: usize;
        core::arch::asm!("ecall", in("x17") ($id),
            in("x10") ($a0), in("x11") ($a1), in("x12") ($a2), in("x13") ($a3),
            lateout("x10") ret);
        ret
    }},
}

pub fn console_putchar(ch: i32) {
    unsafe {
        ecall!(CONSOLE_PUTCHAR, ch);
    }
}

pub fn clear_ipi() {
    unsafe {
        ecall!(CLEAR_IPI);
    }
}

pub fn send_ipi(hart_mask: *const usize) {
    unsafe {
        ecall!(SEND_IPI, hart_mask);
    }
}

pub fn remote_fence_i(hart_mask: *const usize) {
    unsafe {
        ecall!(REMOTE_FENCE_I, hart_mask);
    }
}

pub fn remote_sfence_vma(hart_mask: *const usize, start: usize, size: usize) {
    unsafe {
        ecall!(REMOTE_SFENCE_VMA, hart_mask, start, size);
    }
}

pub fn remote_sfence_vma_asid(hart_mask: *const usize, start: usize, size: usize, asid: usize) {
    unsafe {
        ecall!(REMOTE_SFENCE_VMA_ASID, hart_mask, start, size, asid);
    }
}

pub fn shutdown() {
    unsafe {
        ecall!(SHUTDOWN);
    }
}
