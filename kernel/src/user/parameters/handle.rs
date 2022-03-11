use crate::prelude::*;

impl_errno!(ARGUMENT_HANDLE_NOT_FOUND, 0x83342d52u32);
impl_errno!(ARGUMENT_HANDLE_WRONG_TYPE, 0x2d5c40edu32);

impl Parameter for Handle {
    fn from_arguments(env: &Environment, [x]: [usize; Self::N]) -> EffSys<Self> {
        let handle = env
            .process
            .handle_set
            .lookup(x)
            .ok_or(Errno::ARGUMENT_HANDLE_NOT_FOUND)?;
        Ok(handle)
    }
}

impl<T: Object> Parameter for Handle<T> {
    fn from_arguments(env: &Environment, [x]: [usize; Self::N]) -> EffSys<Self> {
        let handle = env
            .process
            .handle_set
            .lookup(x)
            .ok_or(Errno::ARGUMENT_HANDLE_NOT_FOUND)?;
        Ok(handle.downcast().ok_or(Errno::ARGUMENT_HANDLE_WRONG_TYPE)?)
    }
}
