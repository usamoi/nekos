use crate::prelude::*;

impl_syscall!(DEBUG_WRITE, 0xfbdfbec6u32);
impl_errno!(DEBUG_WRITE_INVAILD_STRING, 0xb29df17du32);

#[async_trait::async_trait]
impl Syscalls<{ Syscall::DEBUG_WRITE }> for Syscall {
    type Arg0 = VAddr;
    type Arg1 = usize;
    async fn syscall(
        env: &Environment,
        (buffer_addr, buffer_len, ..): Self::Args,
    ) -> EffSys<isize> {
        let mut buffer = vec![0u8; buffer_len].into_boxed_slice();
        env.process
            .space
            .read_buffer(buffer_addr, &mut buffer)
            .unwrap();
        let o = core::str::from_utf8(&buffer).map_err(|_| Errno::DEBUG_WRITE_INVAILD_STRING)?;
        print!("{}", o);
        Ok(0)
    }
}

impl_syscall!(DEBUG_EXIT, 0x5a76e1f5u32);

#[async_trait::async_trait]
impl Syscalls<{ Syscall::DEBUG_EXIT }> for Syscall {
    type Arg0 = usize;
    async fn syscall(env: &Environment, (exit_code, ..): Self::Args) -> EffSys<isize> {
        env.thread_exit(exit_code as isize)
            .await
            .map_err(EffectSys::EffectKill)?;
    }
}

impl_syscall!(DEBUG_YIELD, 0x40caac6bu32);

#[async_trait::async_trait]
impl Syscalls<{ Syscall::DEBUG_YIELD }> for Syscall {
    async fn syscall(env: &Environment, (..): Self::Args) -> EffSys<isize> {
        env.thread_yield().await;
        Ok(0)
    }
}
