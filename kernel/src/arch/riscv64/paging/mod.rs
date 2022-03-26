mod errors;
pub use self::errors::*;

cfg_if::cfg_if! {
    if #[cfg(riscv64_paging = "sv39")] {
        mod sv39;
        pub use self::sv39::*;
    } else if #[cfg(riscv64_paging = "sv48")] {
        mod sv48;
        pub use self::sv48::*;
    } else {
        compile_error!("unknown paging");
    }
}

use crate::prelude::*;
use mem::frames::PhysBox;
use riscv::register::satp;

#[repr(transparent)]
#[derive(Debug, Clone, Copy)]
struct PageTableEntry(usize);

impl PageTableEntry {
    const fn new() -> PageTableEntry {
        PageTableEntry(0)
    }
    const fn new_inode(addr: PAddrAligned<12>) -> PageTableEntry {
        PageTableEntry(1 | (addr.into().to_usize() >> 12) << 10)
    }
    const fn new_leaf(
        paddr: PAddr,
        permission: MapPermission,
        user: bool,
        global: bool,
    ) -> PageTableEntry {
        let pte = 1
            | (u8::from(permission) as usize) << 1
            | (user as usize) << 4
            | (global as usize) << 5
            | (paddr.to_usize() >> 12) << 10;
        PageTableEntry(pte)
    }
    const fn is_inode(&self) -> bool {
        !self.get_read() && !self.get_write() && !self.get_execute()
    }
    const fn get_valid(&self) -> bool {
        (self.0 >> 0) & 1 != 0
    }
    const fn get_read(&self) -> bool {
        (self.0 >> 1) & 1 != 0
    }
    const fn get_write(&self) -> bool {
        (self.0 >> 2) & 1 != 0
    }
    const fn get_execute(&self) -> bool {
        (self.0 >> 3) & 1 != 0
    }
    #[allow(dead_code)]
    const fn get_user(&self) -> bool {
        (self.0 >> 4) & 1 != 0
    }
    #[allow(dead_code)]
    const fn get_global(&self) -> bool {
        (self.0 >> 5) & 1 != 0
    }
    #[allow(dead_code)]
    const fn get_accessed(&self) -> bool {
        (self.0 >> 6) & 1 != 0
    }
    #[allow(dead_code)]
    const fn get_dirty(&self) -> bool {
        (self.0 >> 7) & 1 != 0
    }
    #[allow(dead_code)]
    const fn get_rsw0(&self) -> bool {
        (self.0 >> 8) & 1 != 0
    }
    #[allow(dead_code)]
    const fn get_rsw1(&self) -> bool {
        (self.0 >> 9) & 1 != 0
    }
    const fn get_addr(&self) -> PAddrAligned<12> {
        // 52 bit physical address
        PAddrAligned::new(PAddr::new(((self.0 >> 10) & 0xFFFFFFFFFFF) << 12)).unwrap()
    }
}

#[repr(C, align(4096))]
#[derive(Debug, Clone, Index, IndexMut)]
struct PageTableFrame([PageTableEntry; 512]);

impl PageTableFrame {
    const fn new() -> PageTableFrame {
        PageTableFrame([PageTableEntry::new(); 512])
    }
}

unsafe fn find(root: &PhysBox<PageTableFrame>, vpns: &[usize]) -> *mut PageTableEntry {
    assert!(!vpns.is_empty());
    let mut child = &mut (*root.get())[vpns[0]];
    for idx in vpns.iter().copied().skip(1) {
        assert!(child.get_valid(), "Overlapping");
        assert!(child.is_inode(), "Overlapping");
        child = &mut (*child.get_addr().to_mut::<PageTableFrame>())[idx];
    }
    child
}

unsafe fn alloc(root: &PhysBox<PageTableFrame>, vpns: &[usize]) -> *mut PageTableEntry {
    assert!(!vpns.is_empty());
    let mut child = &mut (*root.get())[vpns[0]];
    for idx in vpns.iter().copied().skip(1) {
        if !child.get_valid() {
            let addr =
                PAddrAligned::new(PhysBox::new(PageTableFrame::new()).unwrap().into_raw()).unwrap();
            *child = PageTableEntry::new_inode(addr);
        }
        assert!(child.is_inode(), "Overlapping");
        child = &mut (*child.get_addr().to_mut::<PageTableFrame>())[idx];
    }
    child
}

unsafe fn maintain(root: &PhysBox<PageTableFrame>, mut vpns: &[usize]) {
    while !vpns.is_empty() {
        let child = &mut *find(root, vpns);
        assert!(child.get_valid());
        assert!(child.is_inode());
        let ptf = &mut *child.get_addr().to_mut::<PageTableFrame>();
        for pte in ptf.0.iter() {
            if pte.get_valid() {
                return;
            }
        }
        PhysBox::<PageTableFrame>::from_raw(child.get_addr().into());
        *child = PageTableEntry::new();
        vpns = &vpns[..vpns.len() - 1];
    }
}

#[repr(transparent)]
#[derive(Debug, Clone, Copy)]
pub struct PageTableToken(usize);

impl PageTableToken {
    pub const fn new() -> PageTableToken {
        PageTableToken(0)
    }
    pub unsafe fn switch(self) {
        satp::write(self.0);
        core::arch::riscv64::sfence_vma_all();
        core::arch::riscv64::fence_i();
    }
    pub const fn into_raw(self) -> usize {
        self.0
    }
}

pub struct Template(PageTableFrame);

impl Template {
    pub fn new() -> Template {
        Template(PageTableFrame(array_init::array_init(|i| {
            if i == 511 {
                let addr =
                    PAddrAligned::new(PhysBox::new(PageTableFrame::new()).unwrap().into_raw())
                        .unwrap();
                return PageTableEntry::new_inode(addr);
            }
            PageTableEntry::new()
        })))
    }
}
