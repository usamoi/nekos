use crate::prelude::*;
use proc::process::Process;

impl Object for Process {}

impl_syscall!(PROCESS_CREATE, 0x635e36ceu32);

#[async_trait::async_trait]
impl Syscalls<{ Syscall::PROCESS_CREATE }> for Syscall {
    type Arg0 = usize;
    async fn syscall(env: &Environment, (program_name, ..): Self::Args) -> EffSys<isize> {
        let process = Process::create(program_name).unwrap();
        let id = env.process.handle_set.push(Handle::new(process));
        Ok(id as isize)
    }
}

impl_syscall!(PROCESS_KILL, 0x5050fe08u32);

#[async_trait::async_trait]
impl Syscalls<{ Syscall::PROCESS_KILL }> for Syscall {
    type Arg0 = Handle<Process>;
    type Arg1 = usize;
    async fn syscall(_: &Environment, (process, exit_code, ..): Self::Args) -> EffSys<isize> {
        use ProcessDeath::*;
        process.stop(Exited(exit_code as isize)).unwrap();
        Ok(0)
    }
}
