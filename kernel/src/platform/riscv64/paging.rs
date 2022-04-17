use crate::prelude::*;
use core::fmt::Debug;
use mem::frames::FramesBox;
use rt::paging::*;
use spin::Mutex;

#[repr(transparent)]
#[derive(Debug, Clone, Copy)]
struct PagingEntry(usize);

impl PagingEntry {
    const fn new() -> PagingEntry {
        PagingEntry(0)
    }
    const fn new_inode(addr: PAddr) -> PagingEntry {
        PagingEntry(1 | (addr.to_usize() >> 12) << 10)
    }
    const fn new_leaf(
        paddr: PAddr,
        permission: Permission,
        user: bool,
        global: bool,
    ) -> PagingEntry {
        let pte = 1
            | (permission.as_u8() as usize) << 1
            | (user as usize) << 4
            | (global as usize) << 5
            | (paddr.to_usize() >> 12) << 10;
        PagingEntry(pte)
    }
    const fn valid(&self) -> bool {
        (self.0 >> 0) & 1 != 0
    }
    const fn next(&self) -> bool {
        (self.0 >> 1) & 7 == 0
    }
    const fn addr(&self) -> PAddr {
        // 52 bit physical address
        PAddr::new(((self.0 >> 10) & 0xFFFFFFFFFFF) << 12)
    }
}

#[repr(C, align(4096))]
#[derive(Debug, Clone, Index, IndexMut)]
struct PagingFrame([PagingEntry; 512]);

impl PagingFrame {
    pub const fn new() -> PagingFrame {
        PagingFrame([PagingEntry::new(); 512])
    }
}

unsafe fn find(root: &FramesBox<PagingFrame>, vpns: &[usize]) -> *mut PagingEntry {
    assert!(!vpns.is_empty());
    let mut child = &mut (*root.get())[vpns[0]];
    for idx in vpns.iter().copied().skip(1) {
        assert!(child.valid(), "Overlapping");
        assert!(child.next(), "Overlapping");
        child = &mut (*(child.addr().to_mut() as *mut PagingFrame))[idx];
    }
    child
}

unsafe fn alloc(root: &FramesBox<PagingFrame>, vpns: &[usize]) -> *mut PagingEntry {
    assert!(!vpns.is_empty());
    let mut child = &mut (*root.get())[vpns[0]];
    for idx in vpns.iter().copied().skip(1) {
        if !child.valid() {
            let addr = FramesBox::new(PagingFrame::new()).unwrap().into_raw();
            *child = PagingEntry::new_inode(addr);
        }
        assert!(child.next(), "Overlapping");
        child = &mut (*(child.addr().to_mut() as *mut PagingFrame))[idx];
    }
    child
}

unsafe fn maintain(root: &FramesBox<PagingFrame>, mut vpns: &[usize]) {
    while !vpns.is_empty() {
        let child = &mut *find(root, vpns);
        assert!(child.valid());
        assert!(child.next());
        let ptf = &mut *(child.addr().to_mut() as *mut PagingFrame);
        for pte in ptf.0.iter() {
            if pte.valid() {
                return;
            }
        }
        FramesBox::<PagingFrame>::from_raw(child.addr());
        *child = PagingEntry::new();
        vpns = &vpns[..vpns.len() - 1];
    }
}

core::arch::global_asm!(include_str!("sv39.asm"));

const fn resolve(addr: VAddr) -> Option<([usize; 3], usize)> {
    let addr = addr.to_usize();
    let valid = (addr as isize).wrapping_shl(64 - 39).wrapping_shr(64 - 39);
    if addr != valid as usize {
        return None;
    }
    Some((
        [addr >> 30 & 0x1ff, addr >> 21 & 0x1ff, addr >> 12 & 0x1ff],
        addr & 0xfff,
    ))
}

#[no_mangle]
static mut GLOBAL: PagingFrame = PagingFrame::new();

extern "C" {
    #[link_name = "GLOBAL"]
    static GLOBAL_SYMBOL: Symbol;
}

struct RawPagingInner {
    root: FramesBox<PagingFrame>,
}

impl RawPagingInner {
    fn new() -> RawPagingInner {
        let root = FramesBox::new(PagingFrame([PagingEntry::new(); 512])).unwrap();
        unsafe {
            (*root.get())[511] = PagingEntry::new_inode(super::startup::address(&GLOBAL_SYMBOL));
        }
        RawPagingInner { root }
    }
    fn token(&mut self) -> usize {
        0b1000usize << 60 | self.root.paddr().to_usize() >> 12
    }
    fn map(
        &mut self,
        vaddr: VAddr,
        paddr: PAddr,
        align: usize,
        permission: Permission,
        user: bool,
        global: bool,
    ) -> Result<(), PagingMapError> {
        use PagingMapError::*;
        if !P::check_permission(permission) {
            return Err(PermissionNotSupported);
        }
        if vaddr.to_usize() & (align - 1) != 0 || resolve(vaddr).is_none() {
            return Err(InvalidVAddr);
        }
        if paddr.to_usize() & (align - 1) != 0 || paddr.to_usize() >= (1usize << 52) {
            return Err(InvalidPAddr);
        }
        let ([p3, p2, p1], offset) = resolve(vaddr).unwrap();
        assert!(offset == 0);
        if align == 4 * 1024 {
            unsafe {
                let pte = alloc(&self.root, &[p3, p2, p1]);
                assert!(!(*pte).valid(), "Overlapping");
                pte.write(PagingEntry::new_leaf(paddr, permission, user, global));
                return Ok(());
            }
        }
        if align == 2 * 1024 * 1024 {
            unsafe {
                let pte = alloc(&self.root, &[p3, p2]);
                assert!(!(*pte).valid(), "Overlapping");
                pte.write(PagingEntry::new_leaf(paddr, permission, user, global));
                return Ok(());
            }
        }
        if align == 1024 * 1024 * 1024 {
            unsafe {
                let pte = alloc(&self.root, &[p3]);
                assert!(!(*pte).valid(), "Overlapping");
                pte.write(PagingEntry::new_leaf(paddr, permission, user, global));
                return Ok(());
            }
        }
        Err(AlignNotSupported)
    }
    fn unmap(&mut self, vaddr: VAddr, align: usize) -> Result<PAddr, PagingUnmapError> {
        use PagingUnmapError::*;
        if vaddr.to_usize() & (align - 1) != 0 || resolve(vaddr).is_none() {
            return Err(InvalidVAddr);
        }
        let ([p3, p2, p1], offset) = resolve(vaddr).unwrap();
        assert!(offset == 0);
        if align == 4 * 1024 {
            unsafe {
                let pte = find(&self.root, &[p3, p2, p1]);
                assert!((*pte).valid(), "Overlapping");
                let paddr = (*pte).addr();
                pte.write_volatile(PagingEntry::new());
                maintain(&self.root, &[p3, p2]);
                return Ok(paddr);
            }
        }
        if align == 2 * 1024 * 1024 {
            unsafe {
                let pte = find(&self.root, &[p3, p2]);
                assert!((*pte).valid(), "Overlapping");
                let paddr = (*pte).addr();
                pte.write_volatile(PagingEntry::new());
                maintain(&self.root, &[p3]);
                return Ok(paddr);
            }
        }
        if align == 1024 * 1024 * 1024 {
            unsafe {
                let pte = find(&self.root, &[p3]);
                assert!((*pte).valid(), "Overlapping");
                let paddr = (*pte).addr();
                pte.write_volatile(PagingEntry::new());
                maintain(&self.root, &[]);
                return Ok(paddr);
            }
        }
        Err(AlignNotSupported)
    }
}

pub struct RawPaging {
    inner: Mutex<RawPagingInner>,
}

impl RawPaging {
    pub(in crate::platform) fn token(&self) -> usize {
        let mut inner = self.inner.lock();
        inner.token()
    }
}

impl Paging for RawPaging {
    fn new() -> Self {
        Self {
            inner: Mutex::new(RawPagingInner::new()),
        }
    }
    fn map(
        &self,
        vaddr: VAddr,
        paddr: PAddr,
        align: usize,
        permission: Permission,
        user: bool,
        global: bool,
    ) -> Result<(), PagingMapError> {
        let mut inner = self.inner.lock();
        inner.map(vaddr, paddr, align, permission, user, global)
    }
    fn unmap(&self, vaddr: VAddr, align: usize) -> Result<PAddr, PagingUnmapError> {
        let mut inner = self.inner.lock();
        inner.unmap(vaddr, align)
    }
}

impl Debug for RawPaging {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("RawPaging").finish()
    }
}
