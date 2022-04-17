use crate::prelude::*;
use proc::vmm::*;
use user::objects::memory::Memory;

impl Object for Area {}

impl_syscall!(AREA_CREATE, 0x7d81755fu32);

#[repr(u8)]
pub enum AreaCreateError {
    OutOfRange,
    ZeroSize,
    Overlapping,
}

impl SyscallError for AreaCreateError {
    fn into_u8(self) -> u8 {
        self as u8
    }
}

#[async_trait::async_trait]
impl Syscalls<{ Syscall::AREA_CREATE }> for Syscall {
    type Domain0 = Handle<Area>;
    type Domain1 = VAddr;
    type Domain2 = usize;
    type Codomain = usize;
    type Error = AreaCreateError;
    async fn syscall(env: &Environment, (area, addr, size, ..): domain!()) -> codomain!() {
        use proc::vmm::AreaCreateError as A;
        use AreaCreateError::*;
        let child = area.create(addr, size).map_err(|e| match e {
            A::ZeroSize => ZeroSize,
            A::OutOfRange => OutOfRange,
            A::Overlapping => Overlapping,
        })?;
        let handle_id = env.process.handle_set.push(Handle::new(child));
        Flow::Ok(handle_id)
    }
}

impl_syscall!(AREA_FIND_CREATE, 0x261faebcu32);

#[repr(u8)]
pub enum AreaFindCreateError {
    InvaildLayout,
    ZeroSize,
    OutOfRange,
    OutOfVirtualMemory,
}

impl SyscallError for AreaFindCreateError {
    fn into_u8(self) -> u8 {
        self as u8
    }
}

#[async_trait::async_trait]
impl Syscalls<{ Syscall::AREA_FIND_CREATE }> for Syscall {
    type Domain0 = Handle<Area>;
    type Domain1 = usize;
    type Domain2 = usize;
    type Codomain = usize;
    type Error = AreaFindCreateError;
    async fn syscall(env: &Environment, (area, size, align, ..): domain!()) -> codomain!() {
        use proc::vmm::AreaFindCreateError as E;
        use AreaFindCreateError::*;
        let layout = MapLayout::new(size, align).ok_or(InvaildLayout)?;
        let new = area.find_create(layout).map_err(|e| match e {
            E::ZeroSize => ZeroSize,
            E::OutOfRange => OutOfRange,
            E::OutOfVirtualMemory => OutOfVirtualMemory,
        })?;
        let handle_id = env.process.handle_set.push(Handle::new(new));
        Flow::Ok(handle_id)
    }
}

impl_syscall!(AREA_MAP, 0x4e552567u32);

#[repr(u8)]
pub enum AreaMapError {
    ZeroSize,
    OutOfRange,
    Overlapping,
    BadAddress,
    AlignNotSupported,
    PermissionNotSupported,
}

impl SyscallError for AreaMapError {
    fn into_u8(self) -> u8 {
        self as u8
    }
}

#[async_trait::async_trait]
impl Syscalls<{ Syscall::AREA_MAP }> for Syscall {
    type Domain0 = Handle<Area>;
    type Domain1 = Handle<Memory>;
    type Domain2 = VAddr;
    type Domain3 = Permission;
    type Error = AreaMapError;
    async fn syscall(
        _: &Environment,
        (area, memory, addr, permission, ..): domain!(),
    ) -> codomain!() {
        use proc::vmm::AreaMapError as E;
        use AreaMapError::*;
        area.map(addr, memory.object, permission)
            .map_err(|e| match e {
                E::ZeroSize => ZeroSize,
                E::OutOfRange => OutOfRange,
                E::Overlapping => Overlapping,
                E::BadAddress => BadAddress,
                E::AlignNotSupported => AlignNotSupported,
                E::PermissionNotSupported => PermissionNotSupported,
            })?;
        Flow::Ok(())
    }
}
