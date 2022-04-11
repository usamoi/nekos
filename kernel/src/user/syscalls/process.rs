use crate::prelude::*;
use proc::process::{Process, ProcessStopError};

impl Object for Process {}

impl_syscall!(PROCESS_KILL, 0x5050fe08u32);
impl_errno!(PROCESS_KILL_BAD_STATUS, 0xf79b870au32);

#[async_trait::async_trait]
impl Syscalls<{ Syscall::PROCESS_KILL }> for Syscall {
    type Domain0 = Handle<Process>;
    type Domain1 = usize;
    async fn syscall(
        _: &Environment,
        (process, exit_code, ..): syscall_domain!(),
    ) -> EffSys<Self::Codomain> {
        use ProcessDeath::*;
        use ProcessStopError::*;
        process
            .stop(Exited(exit_code as isize))
            .map_err(|e| match e {
                BadStatus => Errno::PROCESS_KILL_BAD_STATUS,
            })?;
        Ok(())
    }
}
