use crate::prelude::*;

pub fn abort() -> ! {
    P::process_abort();
}
