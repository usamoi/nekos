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
    type Domain0 = Handle<Area>;
    type Domain1 = VAddr;
    type Domain2 = usize;
    type Codomain = usize;
    async fn syscall(
        env: &Environment,
        (area, addr, size, ..): syscall_domain!(),
    ) -> EffSys<Self::Codomain> {
        use AreaCreateError::*;
        let child = area.create(addr, size).map_err(|e| match e {
            ZeroSize => Errno::AREA_CREATE_ZERO_SIZE,
            OutOfRange => Errno::AREA_CREATE_ZERO_SIZE,
            Overlapping => Errno::AREA_CREATE_OVERLAPPING,
        })?;
        let handle_id = env.process.handle_set.push(Handle::new(child));
        Ok(handle_id)
    }
}

impl_syscall!(AREA_FIND_CREATE, 0x261faebcu32);
impl_errno!(AREA_FIND_CREATE_INVALID_LAYOUT, 0x17563d43u32);
impl_errno!(AREA_FIND_CREATE_OUT_OF_RANGE, 0xdba30baau32);
impl_errno!(AREA_FIND_CREATE_ZERO_SIZE, 0xecc00494u32);
impl_errno!(AREA_FIND_CREATE_OOVM, 0x21cca848u32);

#[async_trait::async_trait]
impl Syscalls<{ Syscall::AREA_FIND_CREATE }> for Syscall {
    type Domain0 = Handle<Area>;
    type Domain1 = usize;
    type Domain2 = usize;
    type Codomain = usize;
    async fn syscall(
        env: &Environment,
        (area, size, align, ..): syscall_domain!(),
    ) -> EffSys<Self::Codomain> {
        use AreaFindCreateError::*;
        let layout = MapLayout::new(size, align).ok_or(Errno::AREA_FIND_CREATE_INVALID_LAYOUT)?;
        let new = area.find_create(layout).map_err(|e| match e {
            ZeroSize => Errno::AREA_FIND_CREATE_ZERO_SIZE,
            OutOfRange => Errno::AREA_FIND_CREATE_OUT_OF_RANGE,
            OutOfVirtualMemory => Errno::AREA_FIND_CREATE_OOVM,
        })?;
        let handle_id = env.process.handle_set.push(Handle::new(new));
        Ok(handle_id)
    }
}

impl_syscall!(AREA_MAP, 0x4e552567u32);
impl_errno!(AREA_MAP_ZERO_SIZE, 0x6ae8dadau32);
impl_errno!(AREA_MAP_OUT_OF_RANGE, 0xbe764bddu32);
impl_errno!(AREA_MAP_OVERLAPPING, 0x4d4a2eabu32);
impl_errno!(AREA_MAP_BAD_ADDRESS, 0xb47d1415u32);
impl_errno!(AREA_MAP_ALIGN_NOT_SUPPORTED, 0x1666f6u32);
impl_errno!(AREA_MAP_PERMISSION_NOT_SUPPORTED, 0x66c28fbu32);

#[async_trait::async_trait]
impl Syscalls<{ Syscall::AREA_MAP }> for Syscall {
    type Domain0 = Handle<Area>;
    type Domain1 = Handle<Memory>;
    type Domain2 = VAddr;
    type Domain3 = MapPermission;
    async fn syscall(
        _: &Environment,
        (area, memory, addr, permission, ..): syscall_domain!(),
    ) -> EffSys<Self::Codomain> {
        use AreaMapError::*;
        area.map(addr, memory.object, permission)
            .map_err(|e| match e {
                ZeroSize => Errno::AREA_MAP_ZERO_SIZE,
                OutOfRange => Errno::AREA_MAP_OUT_OF_RANGE,
                Overlapping => Errno::AREA_MAP_OVERLAPPING,
                BadAddress => Errno::AREA_MAP_BAD_ADDRESS,
                AlignNotSupported => Errno::AREA_MAP_ALIGN_NOT_SUPPORTED,
                PermissionNotSupported => Errno::AREA_MAP_PERMISSION_NOT_SUPPORTED,
            })?;
        Ok(())
    }
}

impl_syscall!(AREA_FIND_MAP, 0x13f9d9e7u32);
impl_errno!(AREA_FIND_MAP_ZERO_SIZE, 0x4d8eaa74u32);
impl_errno!(AREA_FIND_MAP_OOVM, 0xd001956fu32);
impl_errno!(AREA_FIND_MAP_ALIGN_NOT_SUPPORTED, 0xcd8ed18eu32);
impl_errno!(AREA_FIND_MAP_PERMISSION_NOT_SUPPORTED, 0x4dd6df50u32);

#[async_trait::async_trait]
impl Syscalls<{ Syscall::AREA_FIND_MAP }> for Syscall {
    type Domain0 = Handle<Area>;
    type Domain1 = Handle<Memory>;
    type Domain2 = MapPermission;
    async fn syscall(
        _: &Environment,
        (area, memory, permission, ..): syscall_domain!(),
    ) -> EffSys<Self::Codomain> {
        use AreaFindMapError::*;
        area.find_map(memory.object, permission)
            .map_err(|e| match e {
                ZeroSize => Errno::AREA_FIND_MAP_ZERO_SIZE,
                OutOfVirtualMemory => Errno::AREA_FIND_MAP_OOVM,
                AlignNotSupported => Errno::AREA_FIND_MAP_ALIGN_NOT_SUPPORTED,
                PermissionNotSupported => Errno::AREA_FIND_MAP_PERMISSION_NOT_SUPPORTED,
            })?;
        Ok(())
    }
}

impl_syscall!(AREA_UNMAP, 0xa9ad74ffu32);
impl_errno!(AREA_UNMAP_UNMAP_AN_AREA, 0x40de67f4u32);
impl_errno!(AREA_UNMAP_NOT_FOUND, 0x79a83b50u32);

#[async_trait::async_trait]
impl Syscalls<{ Syscall::AREA_UNMAP }> for Syscall {
    type Domain0 = Handle<Area>;
    type Domain1 = VAddr;
    async fn syscall(
        _: &Environment,
        (area, addr, ..): syscall_domain!(),
    ) -> EffSys<Self::Codomain> {
        use AreaUnmapError::*;
        area.unmap(addr).map_err(|e| match e {
            UnmapAnArea => Errno::AREA_UNMAP_UNMAP_AN_AREA,
            NotFound => Errno::AREA_UNMAP_NOT_FOUND,
        })?;
        Ok(())
    }
}
