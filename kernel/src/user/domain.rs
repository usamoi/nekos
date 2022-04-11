use crate::prelude::*;

impl Domain for () {
    fn from_arguments(_: &Environment, _: usize) -> EffSys<Self> {
        Ok(())
    }
}

impl Domain for usize {
    fn from_arguments(_: &Environment, x: usize) -> EffSys<Self> {
        Ok(x)
    }
}

impl Domain for isize {
    fn from_arguments(_: &Environment, x: usize) -> EffSys<Self> {
        Ok(x as isize)
    }
}

impl Domain for VAddr {
    fn from_arguments(_: &Environment, x: usize) -> EffSys<Self> {
        Ok(VAddr::new(x))
    }
}

impl_errno!(ARGUMENT_PERMISSION_INVALID, 0x6f3528ebu32);

impl Domain for Permission {
    fn from_arguments(_: &Environment, x: usize) -> EffSys<Self> {
        if x & !0b111 == 0 {
            Ok(Self {
                read: (x & 0b001) != 0,
                write: (x & 0b010) != 0,
                execute: (x & 0b100) != 0,
            })
        } else {
            Err(Errno::ARGUMENT_PERMISSION_INVALID.into())
        }
    }
}

impl_errno!(ARGUMENT_HANDLE_NOT_FOUND, 0x83342d52u32);

impl Domain for Handle {
    fn from_arguments(env: &Environment, x: usize) -> EffSys<Self> {
        let handle = env
            .process
            .handle_set
            .lookup(x)
            .ok_or(Errno::ARGUMENT_HANDLE_NOT_FOUND)?;
        Ok(handle)
    }
}

impl_errno!(ARGUMENT_HANDLET_NOT_FOUND, 0xf111382cu32);
impl_errno!(ARGUMENT_HANDLET_WRONG_TYPE, 0x2d5c40edu32);

impl<T: Object> Domain for Handle<T> {
    fn from_arguments(env: &Environment, x: usize) -> EffSys<Self> {
        let handle = env
            .process
            .handle_set
            .lookup(x)
            .ok_or(Errno::ARGUMENT_HANDLET_NOT_FOUND)?;
        Ok(handle
            .downcast()
            .ok_or(Errno::ARGUMENT_HANDLET_WRONG_TYPE)?)
    }
}
