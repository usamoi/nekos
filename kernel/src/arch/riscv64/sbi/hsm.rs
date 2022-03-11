use super::*;

const EXTENSION_ID: usize = 0x48534D;
const HART_START: usize = 0;
const HART_STOP: usize = 1;
const HART_GET_STATUS: usize = 2;

pub fn hart_start(id: usize, addr: usize, opaque: usize) -> SBIResult {
    unsafe { ecall!(EXTENSION_ID, HART_START, id, addr, opaque) }
}

pub fn hart_stop() -> SBIResult {
    unsafe { ecall!(EXTENSION_ID, HART_STOP) }
}

pub fn hart_get_status(id: usize) -> SBIResult {
    unsafe { ecall!(EXTENSION_ID, HART_GET_STATUS, id) }
}
