use super::TEMPLATE;
use crate::mem::vmm::KArea;
use crate::prelude::*;
use arch::paging::*;
use mem::pages::*;

pub struct KPhys {
    pub segment: Segment<VAddr>,
}

impl KPhys {
    pub fn new() -> KPhys {
        let segment = by_points(
            VAddr::new(0x0000000000000000),
            VAddr::new(0x0000004000000000),
        )
        .unwrap();
        KPhys { segment }
    }
    pub unsafe fn map(&self, page_table: &PageTable) {
        for i in 0..256 {
            let vaddr = VAddr::new(0x40000000 * i);
            let paddr = PAddr::new(0x40000000 * i);
            page_table
                .map(vaddr, paddr, 0x40000000, MapPermission::RW, false, false)
                .unwrap();
        }
    }
}

pub struct KKernel {
    pub segment: Segment<VAddr>,
    pub root: Arc<KArea>,
}

impl KKernel {
    pub fn new(page_table: Arc<PageTable>) -> KKernel {
        let segment = by_points(
            VAddr::new(0xFFFFFFC000000000),
            VAddr::new(0xFFFFFFC040000000),
        )
        .unwrap();
        KKernel {
            segment,
            root: Arc::new(KArea {
                segment,
                page_table,
                page_allocator: Pages::new(segment).unwrap(),
            }),
        }
    }
    pub unsafe fn map(&self, s: usize, e: usize, permission: MapPermission) {
        let vaddr = VAddr::new(s);
        let paddr = config::KERNEL_ADDRESS + (vaddr - _kernel_address.as_vaddr());
        if s != e {
            let layout = MapLayout::new(e - s, 4096).unwrap();
            let map = Arc::new(mem::vmm::KMapUnsafe::new(paddr, layout).unwrap());
            self.root.map(vaddr, map, permission, false).unwrap();
        }
    }
}

pub struct KHeap {
    pub segment: Segment<VAddr>,
}

impl KHeap {
    pub fn new() -> KHeap {
        let segment = by_points(
            VAddr::new(0xFFFFFFC040000000),
            VAddr::new(0xFFFFFFFFC0000000),
        )
        .unwrap();
        KHeap { segment }
    }
}

pub struct KGlobal {
    pub segment: Segment<VAddr>,
    pub root: Arc<KArea>,
}

impl KGlobal {
    pub fn new(page_table: Arc<PageTable>) -> KGlobal {
        let segment = Segment::new(VAddr::new(0xFFFFFFFFC0000000), None).unwrap();
        KGlobal {
            segment,
            root: Arc::new(KArea {
                segment,
                page_table,
                page_allocator: Pages::new(segment).unwrap(),
            }),
        }
    }
}

pub struct KSpace {
    pub page_table: Arc<PageTable>,
    pub phys: KPhys,
    pub kernel: KKernel,
    pub heap: KHeap,
    pub global: KGlobal,
}

impl KSpace {
    pub fn new() -> KSpace {
        let page_table = Arc::new(PageTable::new(&TEMPLATE));
        KSpace {
            phys: KPhys::new(),
            kernel: KKernel::new(page_table.clone()),
            heap: KHeap::new(),
            global: KGlobal::new(page_table.clone()),
            page_table,
        }
    }
}
