use crate::prelude::*;
use proc::process::{Process, ProcessSpawnError};
use proc::thread::Thread;

impl Object for Thread {}

impl_syscall!(THREAD_CREATE, 0x50995b56u32);
impl_errno!(THREAD_CREATE_BAD_STATUS, 0x8f3aa491u32);
impl_errno!(THREAD_CREATE_OOM, 0xd902abafu32);
impl_errno!(THREAD_CREATE_OOVM, 0x3b9a81dbu32);

#[async_trait::async_trait]
impl Syscalls<{ Syscall::THREAD_CREATE }> for Syscall {
    type Domain0 = Handle<Process>;
    type Domain1 = VAddr;
    type Domain2 = usize;
    type Codomain = usize;
    async fn syscall(
        env: &Environment,
        (process, pc, opaque, ..): syscall_domain!(),
    ) -> EffSys<Self::Codomain> {
        use ProcessSpawnError::*;
        let thread = process.spawn(pc, opaque).map_err(|e| match e {
            BadStatus => Errno::THREAD_CREATE_BAD_STATUS,
            OutOfMemory => Errno::THREAD_CREATE_OOM,
            OutOfVirtualMemory => Errno::THREAD_CREATE_OOVM,
        })?;
        let handle_id = env.process.handle_set.push(Handle::new(thread));
        Ok(handle_id)
    }
}

impl_syscall!(THREAD_KILL, 0xf7c12d13u32);

#[async_trait::async_trait]
impl Syscalls<{ Syscall::THREAD_KILL }> for Syscall {
    type Domain0 = Handle<Thread>;
    type Domain1 = usize;
    async fn syscall(
        _: &Environment,
        (thread, exit_code, ..): syscall_domain!(),
    ) -> EffSys<Self::Codomain> {
        thread
            .signal_set
            .send(Signal::KillThread(exit_code as isize));
        Ok(())
    }
}

impl_syscall!(THREAD_YIELD, 0x40caac6bu32);

#[async_trait::async_trait]
impl Syscalls<{ Syscall::THREAD_YIELD }> for Syscall {
    async fn syscall(_: &Environment, (..): syscall_domain!()) -> EffSys<Self::Codomain> {
        yield_now().await;
        Ok(())
    }
}

impl_syscall!(THREAD_EXIT, 0x5a76e1f5u32);

#[async_trait::async_trait]
impl Syscalls<{ Syscall::THREAD_EXIT }> for Syscall {
    type Domain0 = usize;
    async fn syscall(
        env: &Environment,
        (exit_code, ..): syscall_domain!(),
    ) -> EffSys<Self::Codomain> {
        env.thread_exit(exit_code as isize)
            .await
            .map_err(EffectSys::EffectKill)?;
    }
}
