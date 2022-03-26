pub mod codomain;
pub mod defines;
pub mod domain;
pub mod objects;
pub mod syscalls;

use crate::prelude::*;
use common::basic::Is;
use core::mem::MaybeUninit;

async fn solve<const CODE: u32>(env: &Environment, args: Arguments) -> EffSys<usize>
where
    Syscall: Syscalls<CODE>,
    [(); <Syscall as Syscalls<CODE>>::Do0::N]:,
    [(); <Syscall as Syscalls<CODE>>::Do1::N]:,
    [(); <Syscall as Syscalls<CODE>>::Do2::N]:,
    [(); <Syscall as Syscalls<CODE>>::Do3::N]:,
    [(); <Syscall as Syscalls<CODE>>::Do4::N]:,
    [(); <Syscall as Syscalls<CODE>>::Do5::N]:,
{
    fn next_steps<const N: usize, I: Iterator>(this: &mut I) -> Option<[I::Item; N]> {
        let mut ans = [const { MaybeUninit::uninit() }; N];
        for i in 0..N {
            if let Some(x) = this.next() {
                ans[i].write(x);
            } else {
                ans[0..i]
                    .iter_mut()
                    .for_each(|x| unsafe { x.assume_init_drop() });
                return None;
            }
        }
        Some(ans.map(|x| unsafe { x.assume_init() }))
    }
    let mut it = args.into_iter();
    let arg0 = Domain::from_arguments(
        env,
        next_steps(&mut it).ok_or(Errno::GENERAL_NOT_SUPPORTED)?,
    )?;
    let arg1 = Domain::from_arguments(
        env,
        next_steps(&mut it).ok_or(Errno::GENERAL_NOT_SUPPORTED)?,
    )?;
    let arg2 = Domain::from_arguments(
        env,
        next_steps(&mut it).ok_or(Errno::GENERAL_NOT_SUPPORTED)?,
    )?;
    let arg3 = Domain::from_arguments(
        env,
        next_steps(&mut it).ok_or(Errno::GENERAL_NOT_SUPPORTED)?,
    )?;
    let arg4 = Domain::from_arguments(
        env,
        next_steps(&mut it).ok_or(Errno::GENERAL_NOT_SUPPORTED)?,
    )?;
    let arg5 = Domain::from_arguments(
        env,
        next_steps(&mut it).ok_or(Errno::GENERAL_NOT_SUPPORTED)?,
    )?;
    let args = <Syscall as Syscalls<{ CODE }>>::Domain::ID
        .commutative()
        .transport((arg0, arg1, arg2, arg3, arg4, arg5));
    Ok(<Syscall as Syscalls<{ CODE }>>::syscall(env, args)
        .await?
        .to_return_value())
}

impl Environment {
    pub async fn handle_syscall(&self, id: usize, args: Arguments) -> EffSys<usize> {
        match id.try_into().map_err(|_| Errno::GENERAL_INVAILD_SYSCALL)? {
            Syscall::DEBUG_WRITE => solve::<{ Syscall::DEBUG_WRITE }>(self, args).await,
            Syscall::THREAD_EXIT => solve::<{ Syscall::THREAD_EXIT }>(self, args).await,
            Syscall::HANDLE_DROP => solve::<{ Syscall::HANDLE_DROP }>(self, args).await,
            Syscall::PROCESS_CREATE => solve::<{ Syscall::PROCESS_CREATE }>(self, args).await,
            Syscall::PROCESS_KILL => solve::<{ Syscall::PROCESS_KILL }>(self, args).await,
            Syscall::THREAD_CREATE => solve::<{ Syscall::THREAD_CREATE }>(self, args).await,
            Syscall::THREAD_KILL => solve::<{ Syscall::THREAD_KILL }>(self, args).await,
            Syscall::THREAD_YIELD => solve::<{ Syscall::THREAD_YIELD }>(self, args).await,
            Syscall::AREA_CREATE => solve::<{ Syscall::AREA_CREATE }>(self, args).await,
            Syscall::AREA_FIND_CREATE => solve::<{ Syscall::AREA_FIND_CREATE }>(self, args).await,
            Syscall::AREA_MAP => solve::<{ Syscall::AREA_MAP }>(self, args).await,
            Syscall::AREA_FIND_MAP => solve::<{ Syscall::AREA_FIND_MAP }>(self, args).await,
            Syscall::AREA_UNMAP => solve::<{ Syscall::AREA_UNMAP }>(self, args).await,
            _ => Err(Errno::GENERAL_INVAILD_SYSCALL.into()),
        }
    }
}
