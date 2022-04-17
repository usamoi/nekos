use crate::prelude::*;
use base::cell::SingletonCell;

pub struct Vmm {
    pub page_table: Arc<<P as Platform>::Paging>,
    //
    pub global_segment: Segment<VAddr>,
    //
    pub user_segment: Segment<VAddr>,
    //
    pub phys_segment: Segment<VAddr>,
    pub kernel_segment: Segment<VAddr>,
    pub heap_segment: Segment<VAddr>,
}

pub static VMM: SingletonCell<Vmm> = SingletonCell::new();

pub unsafe fn init_global(
    paging: Arc<<P as Platform>::Paging>,
    phys_segment: Segment<VAddr>,
    kernel_segment: Segment<VAddr>,
    heap_segment: Segment<VAddr>,
    global_segment: Segment<VAddr>,
    user_segment: Segment<VAddr>,
) {
    VMM.initialize(Vmm {
        page_table: paging,
        phys_segment,
        kernel_segment,
        heap_segment,
        global_segment,
        user_segment,
    });
}
