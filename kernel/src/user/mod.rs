pub mod defines;
pub mod objects;
pub mod parameters;
pub mod syscalls;
pub mod traits;

use crate::prelude::*;
use common::basic::Is;
use core::mem::MaybeUninit;

async fn solve<const CODE: u32>(env: &Environment, args: Arguments) -> EffSys<isize>
where
    Syscall: Syscalls<CODE>,
    [(); <Syscall as Syscalls<CODE>>::Arg0::N]:,
    [(); <Syscall as Syscalls<CODE>>::Arg1::N]:,
    [(); <Syscall as Syscalls<CODE>>::Arg2::N]:,
    [(); <Syscall as Syscalls<CODE>>::Arg3::N]:,
    [(); <Syscall as Syscalls<CODE>>::Arg4::N]:,
    [(); <Syscall as Syscalls<CODE>>::Arg5::N]:,
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
    let arg0 = Parameter::from_arguments(
        env,
        next_steps(&mut it).ok_or(Errno::GENERAL_NOT_SUPPORTED)?,
    )?;
    let arg1 = Parameter::from_arguments(
        env,
        next_steps(&mut it).ok_or(Errno::GENERAL_NOT_SUPPORTED)?,
    )?;
    let arg2 = Parameter::from_arguments(
        env,
        next_steps(&mut it).ok_or(Errno::GENERAL_NOT_SUPPORTED)?,
    )?;
    let arg3 = Parameter::from_arguments(
        env,
        next_steps(&mut it).ok_or(Errno::GENERAL_NOT_SUPPORTED)?,
    )?;
    let arg4 = Parameter::from_arguments(
        env,
        next_steps(&mut it).ok_or(Errno::GENERAL_NOT_SUPPORTED)?,
    )?;
    let arg5 = Parameter::from_arguments(
        env,
        next_steps(&mut it).ok_or(Errno::GENERAL_NOT_SUPPORTED)?,
    )?;
    let args = <Syscall as Syscalls<{ CODE }>>::Args::ID
        .commutative()
        .transport((arg0, arg1, arg2, arg3, arg4, arg5));
    <Syscall as Syscalls<{ CODE }>>::syscall(env, args).await
}

impl Environment {
    pub async fn handle_syscall(&self, id: usize, args: Arguments) -> EffSys<isize> {
        match id.try_into().map_err(|_| Errno::GENERAL_INVAILD_SYSCALL)? {
            Syscall::DEBUG_WRITE => solve::<{ Syscall::DEBUG_WRITE }>(self, args).await,
            Syscall::DEBUG_EXIT => solve::<{ Syscall::DEBUG_EXIT }>(self, args).await,
            Syscall::DEBUG_YIELD => solve::<{ Syscall::DEBUG_YIELD }>(self, args).await,
            Syscall::HANDLE_DROP => solve::<{ Syscall::HANDLE_DROP }>(self, args).await,
            Syscall::PROCESS_CREATE => solve::<{ Syscall::PROCESS_CREATE }>(self, args).await,
            Syscall::PROCESS_KILL => solve::<{ Syscall::PROCESS_KILL }>(self, args).await,
            Syscall::THREAD_CREATE => solve::<{ Syscall::THREAD_CREATE }>(self, args).await,
            Syscall::THREAD_KILL => solve::<{ Syscall::THREAD_KILL }>(self, args).await,
            Syscall::AREA_CREATE => solve::<{ Syscall::AREA_CREATE }>(self, args).await,
            Syscall::AREA_FIND_CREATE => solve::<{ Syscall::AREA_FIND_CREATE }>(self, args).await,
            Syscall::AREA_MAP => solve::<{ Syscall::AREA_MAP }>(self, args).await,
            Syscall::AREA_FIND_MAP => solve::<{ Syscall::AREA_FIND_MAP }>(self, args).await,
            Syscall::AREA_UNMAP => solve::<{ Syscall::AREA_UNMAP }>(self, args).await,
            Syscall::CHANNEL_CREATE => solve::<{ Syscall::CHANNEL_CREATE }>(self, args).await,
            Syscall::CHANNEL_SEND_BYTES => {
                solve::<{ Syscall::CHANNEL_SEND_BYTES }>(self, args).await
            }
            Syscall::CHANNEL_RECEIVE => solve::<{ Syscall::CHANNEL_RECEIVE }>(self, args).await,
            _ => Err(Errno::GENERAL_INVAILD_SYSCALL.into()),
        }
    }
}
