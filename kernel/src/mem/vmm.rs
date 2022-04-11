use crate::prelude::*;
use base::cell::SingletonCell;
use mem::pages::*;
use rt::paging::Paging;
use rt::paging::PagingGroup;

pub struct KMap {
    paddr: PAddr,
    layout: MapLayout,
}

impl KMap {
    pub unsafe fn new(paddr: PAddr, layout: MapLayout) -> Option<Self> {
        if !layout.check(paddr.to_usize()) {
            return None;
        }
        Some(Self { paddr, layout })
    }
}

impl Map for KMap {
    fn layout(&self) -> MapLayout {
        self.layout
    }
}

impl MapIndex for KMap {
    unsafe fn index_unchecked(&self, i: usize) -> PAddr {
        assert!(i < self.layout.size() / self.layout.align());
        self.paddr + self.layout.align() * i
    }
}

pub struct KArea {
    pub page_table: Arc<<P as Platform>::Paging>,
    pub page_allocator: Pages<Arc<KMap>>,
}

impl KArea {
    pub fn map(&self, vaddr: VAddr, map: Arc<KMap>, permission: Permission, global: bool) {
        assert!(map.layout().check(vaddr.to_usize()));
        assert!(P::paging_align(map.layout().align()));
        assert!(P::paging_permission(permission));
        let segment = by_size(vaddr, map.layout().size()).unwrap();
        self.page_allocator
            .lock()
            .acquire(segment, map.clone())
            .unwrap();
        for i in 0..map.len() {
            let vaddr = segment.start() + i * map.layout().align();
            let paddr = map.index(i);
            self.page_table
                .map(
                    vaddr,
                    paddr,
                    map.layout().align(),
                    permission,
                    false,
                    global,
                )
                .unwrap();
        }
    }
    pub fn find_map(&self, map: Arc<KMap>, permission: Permission, global: bool) -> VAddr {
        assert!(P::paging_align(map.layout().align()));
        assert!(P::paging_permission(permission));
        let mut guard = self.page_allocator.lock();
        let segment = guard.find(map.layout()).unwrap();
        guard.acquire(segment, map.clone()).unwrap();
        for i in 0..map.len() {
            let vaddr = segment.start() + i * map.layout().align();
            let paddr = map.index(i);
            self.page_table
                .map(
                    vaddr,
                    paddr,
                    map.layout().align(),
                    permission,
                    false,
                    global,
                )
                .unwrap();
        }
        segment.start()
    }
}

pub struct KSpace {
    pub page_table: Arc<<P as Platform>::Paging>,
    kernel: Arc<KArea>,
    global: Arc<KArea>,
}

impl KSpace {
    pub fn new() -> KSpace {
        let page_table = Arc::new(<P as Platform>::Paging::new(&GROUP));
        KSpace {
            kernel: Arc::new(KArea {
                page_table: page_table.clone(),
                page_allocator: Pages::new(P::PAGING_KERNEL).unwrap(),
            }),
            global: Arc::new(KArea {
                page_table: page_table.clone(),
                page_allocator: Pages::new(P::PAGING_GLOBAL).unwrap(),
            }),
            page_table,
        }
    }
    pub fn phys_segment(&self) -> Segment<VAddr> {
        P::PAGING_PHYS
    }
    pub fn kernel_segment(&self) -> Segment<VAddr> {
        P::PAGING_KERNEL
    }
    pub unsafe fn kernel_map(&self, s: usize, e: usize, permission: Permission) {
        let vaddr = VAddr::new(s);
        let paddr = config::KERNEL_ADDRESS + (vaddr - _kernel_address.as_vaddr());
        if s != e {
            let layout = MapLayout::new(e - s, 4096).unwrap();
            let map = Arc::new(mem::vmm::KMap::new(paddr, layout).unwrap());
            self.kernel.map(vaddr, map, permission, false);
        }
    }
    pub fn heap_segment(&self) -> Segment<VAddr> {
        P::PAGING_HEAP
    }
    pub fn global_segment(&self) -> Segment<VAddr> {
        P::PAGING_GLOBAL
    }
    pub fn global_map(&self, map: Arc<KMap>, permission: Permission) -> VAddr {
        self.global.find_map(map, permission, true)
    }
}

pub static GROUP: SingletonCell<<P as Platform>::PagingGroup> = SingletonCell::new();
pub static SPACE: SingletonCell<KSpace> = SingletonCell::new();

extern "C" {
    static _text_start: LinkerSymbol;
    static _text_end: LinkerSymbol;
    static _rodata_start: LinkerSymbol;
    static _rodata_end: LinkerSymbol;
    static _tdata_start: LinkerSymbol;
    static _tdata_end: LinkerSymbol;
    static _data_start: LinkerSymbol;
    static _data_end: LinkerSymbol;
    static _bss_start: LinkerSymbol;
    static _bss_end: LinkerSymbol;
    static _uninit_start: LinkerSymbol;
    static _uninit_end: LinkerSymbol;
    static _guard_start: LinkerSymbol;
    static _guard_end: LinkerSymbol;
}

pub unsafe fn init_global() {
    GROUP.initialize(<P as Platform>::PagingGroup::new());
    let space = KSpace::new();
    let memory = rt::mem::memory();
    P::paging_phys(&space.page_table);
    space.kernel_map(_text_start.as_usize(), _text_end.as_usize(), Permission::EO);
    space.kernel_map(
        _rodata_start.as_usize(),
        _rodata_end.as_usize(),
        Permission::RO,
    );
    space.kernel_map(
        _tdata_start.as_usize(),
        _tdata_end.as_usize(),
        Permission::RO,
    );
    space.kernel_map(_data_start.as_usize(), _data_end.as_usize(), Permission::RW);
    space.kernel_map(_bss_start.as_usize(), _bss_end.as_usize(), Permission::RW);
    space.kernel_map(
        _uninit_start.as_usize(),
        _uninit_end.as_usize(),
        Permission::RW,
    );
    space.kernel_map(
        _guard_start.as_usize(),
        (_kernel_address.as_vaddr() + (memory.ptr.to_usize() - memory.start.to_usize())).to_usize(),
        Permission::RW,
    );
    SPACE.initialize(space);
    pt().switch();
}

pub fn pt() -> rt::paging::PagingToken {
    SPACE.page_table.token()
}
