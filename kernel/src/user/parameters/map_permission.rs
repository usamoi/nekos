use crate::prelude::*;

impl_errno!(ARGUMENT_PERMISSION_INVAILD, 0x6f3528ebu32);

impl Parameter for MapPermission {
    fn from_arguments(_: &Environment, [value]: [usize; Self::N]) -> EffSys<Self> {
        if value & !0b111 == 0 {
            Ok(Self {
                read: (value & 0b001) != 0,
                write: (value & 0b010) != 0,
                execute: (value & 0b100) != 0,
            })
        } else {
            Err(Errno::ARGUMENT_PERMISSION_INVAILD.into())
        }
    }
}
