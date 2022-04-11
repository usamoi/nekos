pub mod codomain;
pub mod defines;
pub mod domain;
pub mod objects;
pub mod syscalls;

use crate::prelude::*;

async fn solve<const CODE: u32>(env: &Environment, args: Arguments) -> EffSys<usize>
where
    Syscall: Syscalls<CODE>,
{
    let arg0 = Domain::from_arguments(env, args[0])?;
    let arg1 = Domain::from_arguments(env, args[1])?;
    let arg2 = Domain::from_arguments(env, args[2])?;
    let arg3 = Domain::from_arguments(env, args[3])?;
    let arg4 = Domain::from_arguments(env, args[4])?;
    let arg5 = Domain::from_arguments(env, args[5])?;
    let args = (arg0, arg1, arg2, arg3, arg4, arg5);
    Ok(<Syscall as Syscalls<{ CODE }>>::syscall(env, args)
        .await?
        .to_return_value())
}

impl Environment {
    pub async fn handle_syscall(&self, id: usize, args: Arguments) -> EffSys<usize> {
        match id.try_into().map_err(|_| Errno::GENERAL_INVALID_SYSCALL)? {
            Syscall::DEBUG_WRITE => solve::<{ Syscall::DEBUG_WRITE }>(self, args).await,
            Syscall::THREAD_EXIT => solve::<{ Syscall::THREAD_EXIT }>(self, args).await,
            Syscall::HANDLE_DROP => solve::<{ Syscall::HANDLE_DROP }>(self, args).await,
            Syscall::PROCESS_KILL => solve::<{ Syscall::PROCESS_KILL }>(self, args).await,
            Syscall::THREAD_CREATE => solve::<{ Syscall::THREAD_CREATE }>(self, args).await,
            Syscall::THREAD_KILL => solve::<{ Syscall::THREAD_KILL }>(self, args).await,
            Syscall::THREAD_YIELD => solve::<{ Syscall::THREAD_YIELD }>(self, args).await,
            Syscall::AREA_CREATE => solve::<{ Syscall::AREA_CREATE }>(self, args).await,
            Syscall::AREA_FIND_CREATE => solve::<{ Syscall::AREA_FIND_CREATE }>(self, args).await,
            Syscall::AREA_MAP => solve::<{ Syscall::AREA_MAP }>(self, args).await,
            Syscall::AREA_FIND_MAP => solve::<{ Syscall::AREA_FIND_MAP }>(self, args).await,
            Syscall::AREA_UNMAP => solve::<{ Syscall::AREA_UNMAP }>(self, args).await,
            _ => Err(Errno::GENERAL_INVALID_SYSCALL.into()),
        }
    }
}
