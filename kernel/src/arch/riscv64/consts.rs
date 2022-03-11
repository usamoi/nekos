use crate::prelude::*;

pub const ABI_STACK_ALIGN: usize = 16;
pub const ABI_STACK_OFFSET: usize = 0;
pub const ABI_ELF_MACHINE: u16 = 243;

pub const PAGING_ALIGN_LOG: [bool; usize::BITS as usize] = arch::paging::PAGING_ALIGN_LOG;
pub const PAGING_PERMISSION: [bool; 8] = {
    let mut ans = [false; 8];
    ans[Into::<u8>::into(MapPermission::RO) as usize] = true;
    ans[Into::<u8>::into(MapPermission::RW) as usize] = true;
    ans[Into::<u8>::into(MapPermission::EO) as usize] = true;
    ans[Into::<u8>::into(MapPermission {
        read: true,
        write: false,
        execute: true,
    }) as usize] = true;
    ans
};

pub const fn is_align_supported(align: usize) -> bool {
    assert!(align.is_power_of_two());
    arch::consts::PAGING_ALIGN_LOG[align.log2() as usize]
}

pub const fn is_permission_supported(permission: MapPermission) -> bool {
    arch::consts::PAGING_PERMISSION[Into::<u8>::into(permission) as usize]
}

pub const SBI_MAX_HARTS_NUMBER: usize = 64;
