use super::*;

const EXTENSION_ID: usize = 0x54494D45;
const SET_TIMER: usize = 0;

pub fn set_timer(time: u64) -> SBIResult {
    unsafe { ecall!(EXTENSION_ID, SET_TIMER, time) }
}
