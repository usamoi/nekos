#![allow(unused_assignments)]

pub type HandleID = u32;

macro ecall {
    ($id: expr) => {{
        use ::core::arch::asm;

        let err: u32;
        let ret: usize;
        asm!(
            "ecall", in("x17") ($id as usize),
            lateout("x10") err, lateout("x11") ret
        );
        if err == 0 {
            Ok(ret)
        } else {
            Err(err)
        }
    }},
    ($id: expr, $a0: expr) => {{
        use ::core::arch::asm;

        let err: u32;
        let ret: usize;
        asm!(
            "ecall", in("x17") ($id as usize), in("x10") $a0,
            lateout("x10") err, lateout("x11") ret
        );
        if err == 0 {
            Ok(ret)
        } else {
            Err(err)
        }
    }},
    ($id: expr, $a0: expr, $a1: expr) => {{
        use ::core::arch::asm;

        let err: u32;
        let ret: usize;
        asm!(
            "ecall", in("x17") ($id as usize), in("x10") $a0, in("x11") $a1,
            lateout("x10") err, lateout("x11") ret
        );
        if err == 0 {
            Ok(ret)
        } else {
            Err(err)
        }
    }},
    ($id: expr, $a0: expr, $a1: expr, $a2: expr) => {{
        use ::core::arch::asm;

        let err: u32;
        let ret: usize;
        asm!(
            "ecall", in("x17") ($id as usize), in("x10") $a0, in("x11") $a1, in("x12") $a2,
            lateout("x10") err, lateout("x11") ret
        );
        if err == 0 {
            Ok(ret)
        } else {
            Err(err)
        }
    }},
    ($id: expr, $a0: expr, $a1: expr, $a2: expr, $a3: expr) => {{
        use ::core::arch::asm;

        let err: u32;
        let ret: usize;
        asm!(
            "ecall", in("x17") ($id as usize), in("x10") $a0, in("x11") $a1, in("x12") $a2, in("x13") $a3,
            lateout("x10") err, lateout("x11") ret
        );
        if err == 0 {
            Ok(ret)
        } else {
            Err(err)
        }
    }},
    ($id: expr, $a0: expr, $a1: expr, $a2: expr, $a3: expr, $a4: expr) => {{
        use ::core::arch::asm;

        let err: u32;
        let ret: usize;
        asm!(
            "ecall", in("x17") ($id as usize), in("x10") $a0, in("x11") $a1, in("x12") $a2, in("x13") $a3, in("x14") $a4,
            lateout("x10") err, lateout("x11") ret
        );
        if err == 0 {
            Ok(ret)
        } else {
            Err(err)
        }
    }},
    ($id: expr, $a0: expr, $a1: expr, $a2: expr, $a3: expr, $a4: expr, $a5: expr) => {{
        use ::core::arch::asm;

        let err: u32;
        let ret: usize;
        asm!(
            "ecall", in("x17") ($id as usize), in("x10") $a0, in("x11") $a1, in("x12") $a2, in("x13") $a3, in("x14") $a4, in("x15") $a5,
            lateout("x10") err, lateout("x11") ret
        );
        if err == 0 {
            Ok(ret)
        } else {
            Err(err)
        }
    }},
}

const DEBUG_WRITE: u32 = 0xfbdfbec6u32;
const THREAD_CREATE: u32 = 0x50995b56u32;

pub fn debug_write(s: &str) -> Result<(), !> {
    match unsafe { ecall!(DEBUG_WRITE, s.as_ptr(), s.len()) } {
        Ok(0) => Ok(()),
        _ => unreachable!(),
    }
}

pub fn thread_create(f: extern "C" fn(opaque: usize) -> !, opaque: usize) -> Result<(), !> {
    match unsafe { ecall!(THREAD_CREATE, 0, f, opaque) } {
        _ => Ok(()),
    }
}
