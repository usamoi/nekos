use crate::prelude::*;

impl_syscall!(DEBUG_WRITE, 0xfbdfbec6u32);

#[repr(u8)]
pub enum DebugWriteError {
    InvaildString,
}

impl SyscallError for DebugWriteError {
    fn into_u8(self) -> u8 {
        self as u8
    }
}

#[async_trait::async_trait]
impl Syscalls<{ Syscall::DEBUG_WRITE }> for Syscall {
    type Domain0 = VAddr;
    type Domain1 = usize;
    type Error = DebugWriteError;
    async fn syscall(env: &Environment, (buffer_addr, buffer_len, ..): domain!()) -> codomain!() {
        use DebugWriteError::*;
        let mut buffer = vec![0u8; buffer_len].into_boxed_slice();
        env.process
            .space
            .read_buffer(buffer_addr, &mut buffer)
            .unwrap();
        let o = core::str::from_utf8(&buffer).map_err(|_| InvaildString)?;
        print!("{}", o);
        Flow::Ok(())
    }
}
