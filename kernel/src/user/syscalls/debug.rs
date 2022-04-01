use crate::prelude::*;

impl_syscall!(DEBUG_WRITE, 0xfbdfbec6u32);
impl_errno!(DEBUG_WRITE_INVALID_STRING, 0xb29df17du32);

#[async_trait::async_trait]
impl Syscalls<{ Syscall::DEBUG_WRITE }> for Syscall {
    type Domain0 = VAddr;
    type Domain1 = usize;
    async fn syscall(
        env: &Environment,
        (buffer_addr, buffer_len, ..): syscall_domain!(),
    ) -> EffSys<Self::Codomain> {
        let mut buffer = vec![0u8; buffer_len].into_boxed_slice();
        env.process
            .space
            .read_buffer(buffer_addr, &mut buffer)
            .unwrap();
        let o = core::str::from_utf8(&buffer).map_err(|_| Errno::DEBUG_WRITE_INVALID_STRING)?;
        print!("{}", o);
        Ok(())
    }
}
