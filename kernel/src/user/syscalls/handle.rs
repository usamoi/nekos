use crate::prelude::*;

impl_syscall!(HANDLE_DROP, 0x9c9113fau32);

#[repr(u8)]
pub enum HandleDropError {
    NotFound,
}

impl SyscallError for HandleDropError {
    fn into_u8(self) -> u8 {
        self as u8
    }
}

#[async_trait::async_trait]
impl Syscalls<{ Syscall::HANDLE_DROP }> for Syscall {
    type Domain0 = HandleID;
    type Error = HandleDropError;
    async fn syscall(env: &Environment, (handle_id, ..): domain!()) -> codomain!() {
        use HandleDropError::*;
        let r = env.process.handle_set.remove(handle_id);
        if r.is_some() {
            Flow::Err(NotFound.into())
        } else {
            Flow::Ok(())
        }
    }
}
