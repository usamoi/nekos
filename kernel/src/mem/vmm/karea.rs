use super::errors::*;
use super::kmap::KMap;
use crate::prelude::*;
use arch::consts::{is_align_supported, is_permission_supported};
use arch::paging::*;
use mem::pages::*;

pub struct KArea {
    pub segment: Segment<VAddr>,
    pub page_table: Arc<PageTable>,
    pub page_allocator: Pages<Either<Arc<KArea>, (Arc<dyn KMap>, MapPermission)>>,
}

impl KArea {
    pub fn create(&self, vaddr: VAddr, size: usize) -> Result<Arc<KArea>, KAreaCreateError> {
        use KAreaCreateError::*;
        let segment = by_size(vaddr, size).ok_or(OutOfRange)?;
        let area = Arc::new(KArea {
            segment,
            page_table: self.page_table.clone(),
            page_allocator: Pages::new(segment)?,
        });
        self.page_allocator
            .lock()
            .acquire(segment, Left(area.clone()))?;
        Ok(area)
    }
    pub fn create_find(&self, layout: MapLayout) -> Result<Arc<KArea>, KAreaFindCreateError> {
        let mut guard = self.page_allocator.lock();
        let segment = guard.find(layout)?;
        let area = Arc::new(KArea {
            segment,
            page_table: self.page_table.clone(),
            page_allocator: Pages::new(segment)?,
        });
        guard
            .acquire(segment, Left(area.clone()))
            .out::<KAreaFindCreateError>()?;
        Ok(area)
    }
    pub fn map(
        &self,
        vaddr: VAddr,
        map: Arc<dyn KMap>,
        permission: MapPermission,
        global: bool,
    ) -> Result<(), KAreaMapError> {
        use KAreaMapError::*;
        ensure!(map.layout().check(vaddr.to_usize()), BadAddress);
        ensure!(is_align_supported(map.layout().align()), AlignNotSupported);
        ensure!(is_permission_supported(permission), PermissionNotSupported);
        let segment = by_size(vaddr, map.layout().size()).ok_or(OutOfRange)?;
        self.page_allocator
            .lock()
            .acquire(segment, Right((map.clone(), permission)))?;
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
        Ok(())
    }
    pub fn find_map(
        &self,
        map: Arc<dyn KMap>,
        permission: MapPermission,
        global: bool,
    ) -> Result<VAddr, KAreaFindMapError> {
        use KAreaFindMapError::*;
        ensure!(is_align_supported(map.layout().align()), AlignNotSupported);
        ensure!(is_permission_supported(permission), PermissionNotSupported);
        let mut guard = self.page_allocator.lock();
        let segment = guard.find(map.layout())?;
        guard
            .acquire(segment, Right((map.clone(), permission)))
            .out::<KAreaFindMapError>()?;
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
        Ok(segment.start())
    }
    pub fn unmap(&self, start: VAddr) -> Result<(), KAreaUnmapError> {
        use KAreaUnmapError::*;
        let mut guard = self.page_allocator.lock();
        ensure!(guard.get(start).ok_or(NotFound)?.is_right(), UnmapAnArea);
        let (map, _) = guard.release(start).unwrap().unwrap_right();
        for i in 0..map.len() {
            let vaddr = start + i * map.layout().align();
            self.page_table.unmap(vaddr, map.layout().align()).unwrap();
        }
        Ok(())
    }
}
