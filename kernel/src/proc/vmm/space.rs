use crate::prelude::*;
use arch::paging::PageTable;
use core::mem::MaybeUninit;
use mem::pages::Pages;
use mem::vmm::TEMPLATE;
use proc::vmm::{Area, AreaReadError, AreaWriteError};

pub struct UserSpace {
    pub root: Arc<Area>,
    pub page_table: Arc<PageTable>,
}

impl UserSpace {
    pub fn new() -> Arc<UserSpace> {
        let page_table = Arc::new(PageTable::new(&TEMPLATE));
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
    pub async fn handle_page_fault(&self, _addr: VAddr, kind: MemoryOperation) -> EffKill<()> {
        self.process_fault(ProcessFault::Segment { op: kind })
            .await
            .map(|x| x)
    }
}
