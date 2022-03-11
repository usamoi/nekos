use super::*;
use spin::Mutex;

pub const PAGING_ALIGN_LOG: [bool; usize::BITS as usize] = {
    let mut ans = [false; usize::BITS as usize];
    ans[12] = true;
    ans[21] = true;
    ans[30] = true;
    ans[39] = true;
    ans
};

const fn resolve(addr: VAddr) -> Option<([usize; 4], usize)> {
    let addr = addr.to_usize();
    let valid = (addr as isize).wrapping_shl(64 - 48).wrapping_shr(64 - 48);
    if addr != valid as usize {
        return None;
    }
    Some((
        [
            addr >> 39 & 0x1ff,
            addr >> 30 & 0x1ff,
            addr >> 21 & 0x1ff,
            addr >> 12 & 0x1ff,
        ],
        addr & 0xfff,
    ))
}

pub struct PageTable {
    flag: Mutex<()>,
    root: PhysBox<PageTableFrame>,
    global: bool,
}

impl PageTable {
    pub fn new(global: bool, template: &Template) -> PageTable {
        let root = PhysBox::new(template.0.clone()).unwrap();
        PageTable {
            flag: Mutex::new(()),
            root,
            global,
        }
    }
    pub fn token(&self) -> PageTableToken {
        assert!(!self.global);
        PageTableToken(0b1001usize << 60 | self.root.paddr().to_usize() >> 12)
    }
    pub fn map(
        &self,
        vaddr: VAddr,
        paddr: PAddr,
        align: usize,
        permission: MapPermission,
        user: bool,
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
        let _guard = self.flag.lock();
        let ([p4, p3, p2, p1], offset) = resolve(vaddr).unwrap();
        assert!(offset == 0);
        if align == 4 * 1024 {
            unsafe {
                let pte = alloc(&self.root, &[p4, p3, p2, p1])?;
                ensure!(!(*pte).get_valid(), Overlapping);
                pte.write(PageTableEntry::new_leaf(
                    paddr,
                    permission,
                    user,
                    self.global,
                ));
                return Ok(());
            }
        }
        if align == 2 * 1024 * 1024 {
            unsafe {
                let pte = alloc(&self.root, &[p4, p3, p2])?;
                ensure!(!(*pte).get_valid(), Overlapping);
                pte.write(PageTableEntry::new_leaf(
                    paddr,
                    permission,
                    user,
                    self.global,
                ));
                return Ok(());
            }
        }
        if align == 1024 * 1024 * 1024 {
            unsafe {
                let pte = alloc(&self.root, &[p4, p3])?;
                ensure!(!(*pte).get_valid(), Overlapping);
                pte.write(PageTableEntry::new_leaf(
                    paddr,
                    permission,
                    user,
                    self.global,
                ));
                return Ok(());
            }
        }
        if align == 512 * 1024 * 1024 * 1024 {
            unsafe {
                let pte = alloc(&self.root, &[p4])?;
                ensure!(!(*pte).get_valid(), Overlapping);
                pte.write(PageTableEntry::new_leaf(
                    paddr,
                    permission,
                    user,
                    self.global,
                ));
                return Ok(());
            }
        }
        Err(AlignNotSupported)
    }
    pub fn unmap(&self, vaddr: VAddr, align: usize) -> Result<PAddr, PageTableUnmapError> {
        use PageTableUnmapError::*;
        if vaddr.to_usize() & (align - 1) != 0 || resolve(vaddr).is_none() {
            return Err(InvalidVAddr);
        }
        let _guard = self.flag.lock();
        let ([p4, p3, p2, p1], offset) = resolve(vaddr).unwrap();
        assert!(offset == 0);
        if align == 4 * 1024 {
            unsafe {
                let pte = find(&self.root, &[p4, p3, p2, p1])?;
                ensure!((*pte).get_valid(), Overlapping);
                let paddr = (*pte).get_addr().into();
                pte.write_volatile(PageTableEntry::new());
                maintain(&self.root, &[p4, p3, p2]);
                return Ok(paddr);
            }
        }
        if align == 2 * 1024 * 1024 {
            unsafe {
                let pte = find(&self.root, &[p4, p3, p2])?;
                ensure!((*pte).get_valid(), Overlapping);
                let paddr = (*pte).get_addr().into();
                pte.write_volatile(PageTableEntry::new());
                maintain(&self.root, &[p4, p3]);
                return Ok(paddr);
            }
        }
        if align == 1024 * 1024 * 1024 {
            unsafe {
                let pte = find(&self.root, &[p4, p3])?;
                ensure!((*pte).get_valid(), Overlapping);
                let paddr = (*pte).get_addr().into();
                pte.write_volatile(PageTableEntry::new());
                maintain(&self.root, &[p4]);
                return Ok(paddr);
            }
        }
        if align == 512 * 1024 * 1024 * 1024 {
            unsafe {
                let pte = find(&self.root, &[p4])?;
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
