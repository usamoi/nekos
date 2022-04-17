mod debug;
mod handle;
mod memmap;
mod memory;
mod process;
mod thread;

use crate::prelude::*;

fn helper<T: Domain>(env: &Environment, arg: usize) -> Flow<T, UserError> {
    T::from_arguments(env, arg).map_err(|e| match e {
        Left(e) => UserError::General(e),
        Right(e) => UserError::Domain {
            code: e.into_u8(),
            order: 0,
        },
    })
}

async fn solve<const CODE: u32>(env: &Environment, args: Arguments) -> Flow<usize, UserError>
where
    Syscall: Syscalls<CODE>,
{
    let a0 = helper::<<Syscall as Syscalls<CODE>>::Domain0>(env, args[0])?;
    let a1 = helper::<<Syscall as Syscalls<CODE>>::Domain1>(env, args[1])?;
    let a2 = helper::<<Syscall as Syscalls<CODE>>::Domain2>(env, args[2])?;
    let a3 = helper::<<Syscall as Syscalls<CODE>>::Domain3>(env, args[3])?;
    let a4 = helper::<<Syscall as Syscalls<CODE>>::Domain4>(env, args[4])?;
    let a5 = helper::<<Syscall as Syscalls<CODE>>::Domain5>(env, args[5])?;
    let a = (a0, a1, a2, a3, a4, a5);
    let b = <Syscall as Syscalls<{ CODE }>>::syscall(env, a)
        .await
        .map_err(|e| match e {
            Left(e) => UserError::General(e),
            Right(e) => UserError::Syscall { code: e.into_u8() },
        })?;
    Flow::Ok(b.to_return_value())
}

impl Environment {
    pub async fn handle_syscall(&self, id: usize, args: Arguments) -> Flow<usize, UserError> {
        match id
            .try_into()
            .map_err(|_| UserError::General(GeneralError::InvaildSyscall))?
        {
            Syscall::DEBUG_WRITE => solve::<{ Syscall::DEBUG_WRITE }>(self, args).await,
            Syscall::THREAD_EXIT => solve::<{ Syscall::THREAD_EXIT }>(self, args).await,
            Syscall::HANDLE_DROP => solve::<{ Syscall::HANDLE_DROP }>(self, args).await,
            Syscall::THREAD_CREATE => solve::<{ Syscall::THREAD_CREATE }>(self, args).await,
            Syscall::THREAD_KILL => solve::<{ Syscall::THREAD_KILL }>(self, args).await,
            Syscall::THREAD_YIELD => solve::<{ Syscall::THREAD_YIELD }>(self, args).await,
            Syscall::AREA_CREATE => solve::<{ Syscall::AREA_CREATE }>(self, args).await,
            Syscall::AREA_FIND_CREATE => solve::<{ Syscall::AREA_FIND_CREATE }>(self, args).await,
            Syscall::AREA_MAP => solve::<{ Syscall::AREA_MAP }>(self, args).await,
            _ => Flow::Err(UserError::General(GeneralError::InvaildSyscall)),
        }
    }
}
