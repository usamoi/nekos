use crate::prelude::*;
use proc::process::{Process, ProcessSpawnError};
use proc::thread::Thread;

impl Object for Thread {}

impl_syscall!(THREAD_CREATE, 0x50995b56u32);

#[repr(u8)]
pub enum SyscallThreadCreateError {
    BadStatus,
    OutOfMemory,
    OutOfVirtualMemory,
}

impl SyscallError for SyscallThreadCreateError {
    fn into_u8(self) -> u8 {
        self as u8
    }
}

#[async_trait::async_trait]
impl Syscalls<{ Syscall::THREAD_CREATE }> for Syscall {
    type Domain0 = Handle<Process>;
    type Domain1 = VAddr;
    type Domain2 = usize;
    type Codomain = usize;
    type Error = SyscallThreadCreateError;
    async fn syscall(env: &Environment, (process, pc, opaque, ..): domain!()) -> codomain!() {
        use ProcessSpawnError as E;
        use SyscallThreadCreateError::*;
        let thread = process.spawn(pc, opaque).map_err(|e| match e {
            E::BadStatus => BadStatus,
            E::OutOfMemory => OutOfMemory,
            E::OutOfVirtualMemory => OutOfVirtualMemory,
        })?;
        let handle_id = env.process.handle_set.push(Handle::new(thread));
        Flow::Ok(handle_id)
    }
}

impl_syscall!(THREAD_KILL, 0xf7c12d13u32);

#[async_trait::async_trait]
impl Syscalls<{ Syscall::THREAD_KILL }> for Syscall {
    type Domain0 = Handle<Thread>;
    type Domain1 = usize;
    type Error = !;
    async fn syscall(_: &Environment, (thread, exit_code, ..): domain!()) -> codomain!() {
        thread
            .signal_set
            .send(Signal::KillThread(exit_code as isize));
        Flow::Ok(())
    }
}

impl_syscall!(THREAD_YIELD, 0x40caac6bu32);

#[async_trait::async_trait]
impl Syscalls<{ Syscall::THREAD_YIELD }> for Syscall {
    type Error = !;
    async fn syscall(_: &Environment, (..): domain!()) -> codomain!() {
        base::future::yield_now().await;
        Flow::Ok(())
    }
}

impl_syscall!(THREAD_EXIT, 0x5a76e1f5u32);

#[async_trait::async_trait]
impl Syscalls<{ Syscall::THREAD_EXIT }> for Syscall {
    type Domain0 = usize;
    type Error = !;
    async fn syscall(env: &Environment, (exit_code, ..): domain!()) -> codomain!() {
        env.thread_exit(exit_code as isize).await?;
    }
}
