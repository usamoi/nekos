use crate::prelude::*;
use user::objects::memory::*;

impl Object for Memory {}

impl_syscall!(MEMORY_CREATE, 0x345fc9e5u32);
impl_errno!(MEMORY_CREATE_INVAILD_LAYOUT, 0x69c9cf35u32);
impl_errno!(MEMORY_CREATE_ZERO_SIZE, 0xd9d14f34u32);
impl_errno!(MEMORY_CREATE_UNDERSIZE_ALIGN, 0xc3cc443au32);
impl_errno!(MEMORY_CREATE_OUT_OF_MEMORY, 0x73e9b871u32);

#[async_trait::async_trait]
impl Syscalls<{ Syscall::MEMORY_CREATE }> for Syscall {
    type Arg0 = usize;
    type Arg1 = usize;
    async fn syscall(env: &Environment, (size, align, ..): Self::Args) -> EffSys<isize> {
        use MemoryCreateError::*;
        let layout = MapLayout::new(size, align).ok_or(Errno::MEMORY_CREATE_INVAILD_LAYOUT)?;
        let memory = Memory::create(layout).map_err(|e| match e {
            ZeroSize => Errno::MEMORY_CREATE_ZERO_SIZE,
            UndersizeAlign => Errno::MEMORY_CREATE_UNDERSIZE_ALIGN,
            OutOfMemory => Errno::MEMORY_CREATE_OUT_OF_MEMORY,
        })?;
        let handle_id = env.process.handle_set.push(Handle::new(memory));
        Ok(handle_id as isize)
    }
}
