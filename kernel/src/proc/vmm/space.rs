use crate::{mem::vmm::VMM, prelude::*};
use mem::pages::Pages;
use proc::vmm::{Area, AreaReadError, AreaWriteError};
use rt::paging::Paging;

pub struct UserSpace {
    pub root: Arc<Area>,
    pub page_table: Arc<<P as Platform>::Paging>,
}

impl UserSpace {
    pub fn new() -> Arc<UserSpace> {
        let page_table = Arc::new(<P as Platform>::Paging::new());
        Arc::new(UserSpace {
            root: Arc::new(Area {
                segment: VMM.user_segment,
                page_table: page_table.clone(),
                page_allocator: Pages::new(VMM.user_segment).unwrap(),
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
}

impl Environment {
    pub async fn handle_page_fault(&self, _addr: VAddr, access: Access) -> Flow<()> {
        self.process_fault(ProcessFault::Segment { access })
            .await
            .map(|x| x)
    }
}
