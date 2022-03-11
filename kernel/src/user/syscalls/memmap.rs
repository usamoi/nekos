use crate::prelude::*;
use proc::vmm::*;
use user::objects::memory::Memory;

impl Object for Area {}

impl_syscall!(AREA_CREATE, 0x7d81755fu32);
impl_errno!(AREA_CREATE_OUT_OF_RANGE, 0x2f70ab08u32);
impl_errno!(AREA_CREATE_ZERO_SIZE, 0xd47ac121u32);
impl_errno!(AREA_CREATE_OVERLAPPING, 0xb4783099u32);

#[async_trait::async_trait]
impl Syscalls<{ Syscall::AREA_CREATE }> for Syscall {
    type Arg0 = Handle<Area>;
    type Arg1 = VAddr;
    type Arg2 = usize;
    async fn syscall(env: &Environment, (area, addr, size, ..): Self::Args) -> EffSys<isize> {
        use AreaCreateError::*;
        let child = area.create(addr, size).map_err(|e| match e {
            ZeroSize => Errno::AREA_CREATE_ZERO_SIZE,
            OutOfRange => Errno::AREA_CREATE_ZERO_SIZE,
            Overlapping => Errno::AREA_CREATE_OVERLAPPING,
        })?;
        let handle_id = env.process.handle_set.push(Handle::new(child));
        Ok(handle_id as isize)
    }
}

impl_syscall!(AREA_FIND_CREATE, 0x261faebcu32);
impl_errno!(AREA_FIND_CREATE_INVAILD_LAYOUT, 0x17563d43u32);
impl_errno!(AREA_FIND_CREATE_OUT_OF_RANGE, 0xdba30baau32);
impl_errno!(AREA_FIND_CREATE_ZERO_SIZE, 0xecc00494u32);
impl_errno!(AREA_FIND_CREATE_OUT_OF_VIRTUAL_MEMORY, 0x21cca848u32);

#[async_trait::async_trait]
impl Syscalls<{ Syscall::AREA_FIND_CREATE }> for Syscall {
    type Arg0 = Handle<Area>;
    type Arg1 = usize;
    type Arg2 = usize;
    async fn syscall(env: &Environment, (area, size, align, ..): Self::Args) -> EffSys<isize> {
        let layout = MapLayout::new(size, align).unwrap();
        let new = area.find_create(layout).unwrap();
        let handle_id = env.process.handle_set.push(Handle::new(new));
        Ok(handle_id as isize)
    }
}

impl_syscall!(AREA_MAP, 0x4e552567u32);

#[async_trait::async_trait]
impl Syscalls<{ Syscall::AREA_MAP }> for Syscall {
    type Arg0 = Handle<Area>;
    type Arg1 = Handle<Memory>;
    type Arg2 = VAddr;
    type Arg3 = MapPermission;
    async fn syscall(
        _: &Environment,
        (area, memory, addr, permission, ..): Self::Args,
    ) -> EffSys<isize> {
        area.map(addr, memory.object, permission).unwrap();
        Ok(0)
    }
}

impl_syscall!(AREA_FIND_MAP, 0x13f9d9e7u32);

#[async_trait::async_trait]
impl Syscalls<{ Syscall::AREA_FIND_MAP }> for Syscall {
    type Arg0 = Handle<Area>;
    type Arg1 = Handle<Memory>;
    type Arg2 = MapPermission;
    async fn syscall(_: &Environment, (area, memory, permission, ..): Self::Args) -> EffSys<isize> {
        area.find_map(memory.object, permission).unwrap();
        Ok(0)
    }
}

impl_syscall!(AREA_UNMAP, 0xa9ad74ffu32);

#[async_trait::async_trait]
impl Syscalls<{ Syscall::AREA_UNMAP }> for Syscall {
    type Arg0 = Handle<Area>;
    type Arg1 = VAddr;
    async fn syscall(_: &Environment, (area, addr, ..): Self::Args) -> EffSys<isize> {
        area.unmap(addr).unwrap();
        Ok(0)
    }
}
