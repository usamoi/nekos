use super::*;
use spin::Mutex;

pub const PAGING_ALIGN_LOG: [bool; usize::BITS as usize] = {
    let mut ans = [false; usize::BITS as usize];
    ans[12] = true;
    ans[21] = true;
    ans[30] = true;
    ans
};

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

struct PageTableMut {
    root: PhysBox<PageTableFrame>,
}

impl PageTableMut {
    fn new(template: &Template) -> PageTableMut {
        let root = PhysBox::new(template.0.clone()).unwrap();
        PageTableMut { root }
    }
    fn token(&mut self) -> PageTableToken {
        PageTableToken(0b1000usize << 60 | self.root.paddr().to_usize() >> 12)
    }
    fn map(
        &mut self,
        vaddr: VAddr,
        paddr: PAddr,
        align: usize,
        permission: MapPermission,
        user: bool,
        global: bool,
    ) -> Result<(), PageTableMapError> {
        use PageTableMapError::*;
        if !arch::consts::is_permission_supported(permission) {
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
                let pte = alloc(&self.root, &[p3, p2, p1])?;
                ensure!(!(*pte).get_valid(), Overlapping);
                pte.write(PageTableEntry::new_leaf(paddr, permission, user, global));
                return Ok(());
            }
        }
        if align == 2 * 1024 * 1024 {
            unsafe {
                let pte = alloc(&self.root, &[p3, p2])?;
                ensure!(!(*pte).get_valid(), Overlapping);
                pte.write(PageTableEntry::new_leaf(paddr, permission, user, global));
                return Ok(());
            }
        }
        if align == 1024 * 1024 * 1024 {
            unsafe {
                let pte = alloc(&self.root, &[p3])?;
                ensure!(!(*pte).get_valid(), Overlapping);
                pte.write(PageTableEntry::new_leaf(paddr, permission, user, global));
                return Ok(());
            }
        }
        Err(AlignNotSupported)
    }
    fn unmap(&mut self, vaddr: VAddr, align: usize) -> Result<PAddr, PageTableUnmapError> {
        use PageTableUnmapError::*;
        if vaddr.to_usize() & (align - 1) != 0 || resolve(vaddr).is_none() {
            return Err(InvalidVAddr);
        }
        let ([p3, p2, p1], offset) = resolve(vaddr).unwrap();
        assert!(offset == 0);
        if align == 4 * 1024 {
            unsafe {
                let pte = find(&self.root, &[p3, p2, p1])?;
                ensure!((*pte).get_valid(), Overlapping);
                let paddr = (*pte).get_addr().into();
                pte.write_volatile(PageTableEntry::new());
                maintain(&self.root, &[p3, p2]);
                return Ok(paddr);
            }
        }
        if align == 2 * 1024 * 1024 {
            unsafe {
                let pte = find(&self.root, &[p3, p2])?;
                ensure!((*pte).get_valid(), Overlapping);
                let paddr = (*pte).get_addr().into();
                pte.write_volatile(PageTableEntry::new());
                maintain(&self.root, &[p3]);
                return Ok(paddr);
            }
        }
        if align == 1024 * 1024 * 1024 {
            unsafe {
                let pte = find(&self.root, &[p3])?;
                ensure!((*pte).get_valid(), Overlapping);
                let paddr = (*pte).get_addr().into();
                pte.write_volatile(PageTableEntry::new());
                maintain(&self.root, &[]);
                return Ok(paddr);
            }
        }
        Err(AlignNotSupported)
    }
}

pub struct PageTable {
    page_table: Mutex<PageTableMut>,
}

impl PageTable {
    pub fn new(template: &Template) -> Self {
        Self {
            page_table: Mutex::new(PageTableMut::new(template)),
        }
    }
    pub fn token(&self) -> PageTableToken {
        let mut inner = self.page_table.lock();
        inner.token()
    }
    pub fn map(
        &self,
        vaddr: VAddr,
        paddr: PAddr,
        align: usize,
        permission: MapPermission,
        user: bool,
        global: bool,
    ) -> Result<(), PageTableMapError> {
        let mut inner = self.page_table.lock();
        inner.map(vaddr, paddr, align, permission, user, global)
    }
    pub fn unmap(&self, vaddr: VAddr, align: usize) -> Result<PAddr, PageTableUnmapError> {
        let mut inner = self.page_table.lock();
        inner.unmap(vaddr, align)
    }
}
