use crate::prelude::*;
use core::mem::MaybeUninit;
use mem::pages::Pages;
use mem::vmm::GROUP;
use proc::vmm::{Area, AreaReadError, AreaWriteError};
use rt::paging::Paging;

pub struct UserSpace {
    pub root: Arc<Area>,
    pub page_table: Arc<<P as Platform>::Paging>,
}

impl UserSpace {
    pub fn new() -> Arc<UserSpace> {
        let page_table = Arc::new(<P as Platform>::Paging::new(&GROUP));
        let segment = by_points(
            VAddr::new(0x0000000000000000),
            VAddr::new(0x0000004000000000),
        )
        .unwrap();
        Arc::new(UserSpace {
            root: Arc::new(Area {
                segment,
                page_table: page_table.clone(),
                page_allocator: Pages::new(segment).unwrap(),
            }),
            page_table,
        })
    }
    pub fn segment(&self) -> Segment<VAddr> {
        self.root.segment
    }
    pub fn read_buffer(&self, addr: VAddr, buffer: &mut [u8]) -> Result<(), AreaReadError> {
        self.root.read(addr, buffer)
    }
    pub fn write_buffer(&self, addr: VAddr, buffer: &[u8]) -> Result<(), AreaWriteError> {
        self.root.write(addr, buffer)
    }
    pub fn read_value<T: Sized + Copy>(&self, addr: VAddr) -> Result<(), AreaReadError> {
        let mut buffer: Box<MaybeUninit<T>> = Box::new_uninit();
        let ptr = unsafe {
            core::slice::from_raw_parts_mut(
                buffer.as_mut_ptr() as *mut u8,
                core::mem::size_of::<T>(),
            )
        };
        self.read_buffer(addr, ptr)
    }
    pub fn write_value<T: ?Sized + Copy>(
        &self,
        addr: VAddr,
        value: T,
    ) -> Result<(), AreaWriteError> {
        let ptr = unsafe {
            core::slice::from_raw_parts(
                core::ptr::addr_of!(value) as *const u8,
                core::mem::size_of_val(&value),
            )
        };
        self.write_buffer(addr, ptr)
    }
}

impl Environment {
    pub async fn handle_page_fault(&self, _addr: VAddr, access: Access) -> EffKill<()> {
        self.process_fault(ProcessFault::Segment { access })
            .await
            .map(|x| x)
    }
}
