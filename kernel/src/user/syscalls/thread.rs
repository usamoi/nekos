use crate::prelude::*;
use proc::process::Process;
use proc::thread::Thread;

impl Object for Thread {}

impl_syscall!(THREAD_CREATE, 0x50995b56u32);

#[async_trait::async_trait]
impl Syscalls<{ Syscall::THREAD_CREATE }> for Syscall {
    type Arg0 = Handle<Process>;
    type Arg1 = VAddr;
    type Arg2 = usize;
    async fn syscall(env: &Environment, (process, pc, opaque, ..): Self::Args) -> EffSys<isize> {
        let thread = process.object.spawn(pc, opaque).unwrap();
        let handle_id = env.process.handle_set.push(Handle::new(thread));
        Ok(handle_id as isize)
    }
}

impl_syscall!(THREAD_KILL, 0xf7c12d13u32);

#[async_trait::async_trait]
impl Syscalls<{ Syscall::THREAD_KILL }> for Syscall {
    type Arg0 = Handle<Thread>;
    type Arg1 = usize;
    async fn syscall(_: &Environment, (thread, exit_code, ..): Self::Args) -> EffSys<isize> {
        thread
            .signal_set
            .send(Signal::KillThread(exit_code as isize));
        Ok(0)
    }
}
