use crate::prelude::*;
use proc::process::{Process, ProcessCreateError, ProcessStopError};

impl Object for Process {}

impl_syscall!(PROCESS_CREATE, 0x635e36ceu32);
impl_errno!(PROCESS_CREATE_LOAD_ERROR, 0x1595a966u32);
impl_errno!(PROCESS_CREATE_OOM, 0xbabfe05cu32);
impl_errno!(PROCESS_CREATE_OOVM, 0xbeef68e6u32);

#[async_trait::async_trait]
impl Syscalls<{ Syscall::PROCESS_CREATE }> for Syscall {
    type Do0 = usize;
    type Codomain = usize;
    async fn syscall(
        env: &Environment,
        (program_name, ..): Self::Domain,
    ) -> EffSys<Self::Codomain> {
        use ProcessCreateError::*;
        let process = Process::create(program_name).map_err(|e| match e {
            LoadError => Errno::PROCESS_CREATE_LOAD_ERROR,
            OutOfMemory => Errno::PROCESS_CREATE_OOM,
            OutOfVirtualMemory => Errno::PROCESS_CREATE_OOVM,
        })?;
        let id = env.process.handle_set.push(Handle::new(process));
        Ok(id)
    }
}

impl_syscall!(PROCESS_KILL, 0x5050fe08u32);
impl_errno!(PROCESS_KILL_BAD_STATUS, 0xf79b870au32);

#[async_trait::async_trait]
impl Syscalls<{ Syscall::PROCESS_KILL }> for Syscall {
    type Do0 = Handle<Process>;
    type Do1 = usize;
    async fn syscall(
        _: &Environment,
        (process, exit_code, ..): Self::Domain,
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
