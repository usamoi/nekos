use super::map::MapUser;
use super::*;
use crate::prelude::*;
use mem::pages::*;
use rt::paging::Paging;

pub struct Area {
    pub segment: Segment<VAddr>,
    pub page_table: Arc<<P as Platform>::Paging>,
    pub page_allocator: Pages<Either<Arc<Area>, (Arc<dyn MapUser>, Permission)>>,
}

impl Area {
    pub fn create(&self, vaddr: VAddr, size: usize) -> Result<Arc<Area>, AreaCreateError> {
        use AreaCreateError::*;
        let segment = by_size(vaddr, size).ok_or(OutOfRange)?;
        let area = Arc::new(Area {
            segment,
            page_table: self.page_table.clone(),
            page_allocator: Pages::new(segment)?,
        });
        self.page_allocator
            .lock()
            .acquire(segment, Left(area.clone()))?;
        Ok(area)
    }
    pub fn find_create(&self, layout: MapLayout) -> Result<Arc<Area>, AreaFindCreateError> {
        let mut guard = self.page_allocator.lock();
        let segment = guard.find(layout)?;
        let area = Arc::new(Area {
            segment,
            page_table: self.page_table.clone(),
            page_allocator: Pages::new(segment)?,
        });
        guard
            .acquire(segment, Left(area.clone()))
            .out::<AreaFindCreateError>()?;
        Ok(area)
    }
    pub fn map(
        &self,
        vaddr: VAddr,
        map: Arc<dyn MapUser>,
        permission: Permission,
    ) -> Result<(), AreaMapError> {
        use AreaMapError::*;
        ensure!(map.layout().check(vaddr.to_usize()), BadAddress);
        ensure!(P::check_align(map.layout().align()), AlignNotSupported);
        ensure!(P::check_permission(permission), PermissionNotSupported);
        let segment = by_size(vaddr, map.layout().size()).ok_or(OutOfRange)?;
        self.page_allocator
            .lock()
            .acquire(segment, Right((map.clone(), permission)))?;
        for i in 0..map.len() {
            let vaddr = segment.start() + i * map.layout().align();
            let paddr = map.index(i);
            self.page_table
                .map(vaddr, paddr, map.layout().align(), permission, true, false)
                .unwrap();
        }
        Ok(())
    }
    pub fn find_map(
        &self,
        map: Arc<dyn MapUser>,
        permission: Permission,
    ) -> Result<VAddr, AreaFindMapError> {
        use AreaFindMapError::*;
        ensure!(P::check_align(map.layout().align()), AlignNotSupported);
        ensure!(P::check_permission(permission), PermissionNotSupported);
        let mut guard = self.page_allocator.lock();
        let segment = guard.find(map.layout())?;
        guard
            .acquire(segment, Right((map.clone(), permission)))
            .out::<AreaFindMapError>()?;
        for i in 0..map.len() {
            let vaddr = segment.start() + i * map.layout().align();
            let paddr = map.index(i);
            self.page_table
                .map(vaddr, paddr, map.layout().align(), permission, true, false)
                .unwrap();
        }
        Ok(segment.start())
    }
    pub fn unmap(&self, start: VAddr) -> Result<(), AreaUnmapError> {
        use AreaUnmapError::*;
        let mut guard = self.page_allocator.lock();
        ensure!(guard.get(start).ok_or(NotFound)?.is_right(), UnmapAnArea);
        let (map, _) = guard.release(start).unwrap().unwrap_right();
        for i in 0..map.len() {
            let vaddr = start + i * map.layout().align();
            self.page_table.unmap(vaddr, map.layout().align()).unwrap();
        }
        Ok(())
    }
    pub fn read(&self, mut addr: VAddr, mut buffer: &mut [u8]) -> Result<(), AreaReadError> {
        use AreaReadError::*;
        let segment = by_size(addr, buffer.len()).ok_or(OutOfRange)?;
        ensure!(self.segment.contains(segment), OutOfRange);
        let guard = self.page_allocator.lock();
        while !buffer.is_empty() {
            let val = guard.locate(addr).ok_or(BadRead)?;
            let len = usize::min(buffer.len(), val.0.wrapping_end() - addr);
            match &val.1 {
                Left(area) => {
                    area.read(addr, &mut buffer[..len])?;
                }
                Right((map, permission)) => {
                    ensure!(permission.read, PermissionDenied);
                    map.read(addr - val.0.start(), &mut buffer[..len]);
                }
            }
            addr = addr + len;
            buffer = &mut buffer[len..];
        }
        Ok(())
    }
    pub fn write(&self, mut addr: VAddr, mut buffer: &[u8]) -> Result<(), AreaWriteError> {
        use AreaWriteError::*;
        let segment = by_size(addr, buffer.len()).ok_or(OutOfRange)?;
        ensure!(self.segment.contains(segment), OutOfRange);
        let guard = self.page_allocator.lock();
        while !buffer.is_empty() {
            let val = guard.locate(addr).ok_or(BadWrite)?;
            let len = usize::min(buffer.len(), val.0.wrapping_end() - addr);
            match &val.1 {
                Left(area) => {
                    area.write(addr, &buffer[..len])?;
                }
                Right((map, permission)) => {
                    ensure!(permission.write, PermissionDenied);
                    map.write(addr - val.0.start(), &buffer[..len]);
                }
            }
            addr = addr + len;
            buffer = &buffer[len..];
        }
        Ok(())
    }
}
