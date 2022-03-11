pub mod handle;
pub mod map_permission;

use crate::prelude::*;

impl Parameter for () {
    const N: usize = 0;

    fn from_arguments(_: &Environment, []: [usize; Self::N]) -> EffSys<Self> {
        Ok(())
    }
}

impl Parameter for usize {
    fn from_arguments(_: &Environment, [x]: [usize; Self::N]) -> EffSys<Self> {
        Ok(x)
    }
}

impl Parameter for isize {
    fn from_arguments(_: &Environment, [x]: [usize; Self::N]) -> EffSys<Self> {
        Ok(x as isize)
    }
}

impl Parameter for VAddr {
    fn from_arguments(_: &Environment, [x]: [usize; Self::N]) -> EffSys<Self> {
        Ok(VAddr::new(x))
    }
}
