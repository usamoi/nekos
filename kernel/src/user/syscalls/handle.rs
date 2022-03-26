use crate::prelude::*;

impl_syscall!(HANDLE_DROP, 0x9c9113fau32);
impl_errno!(HANDLE_DROP_NOT_FOUND, 0xfd3b5c6du32);

#[async_trait::async_trait]
impl Syscalls<{ Syscall::HANDLE_DROP }> for Syscall {
    type Do0 = HandleID;
    async fn syscall(
        env: &Environment,
        (handle_id, ..): (HandleID, (), (), (), (), ()),
    ) -> EffSys<Self::Codomain> {
        let r = env.process.handle_set.remove(handle_id);
        if r.is_some() {
            Err(Errno::HANDLE_DROP_NOT_FOUND.into())
        } else {
            Ok(())
        }
    }
}
