mod buddy;

mod errors;
pub use self::errors::*;

use crate::prelude::*;
use alloc::collections::BTreeMap;
use buddy::Buddy;
use spin::{Mutex, MutexGuard};

fn map(segment: Segment<VAddr>) -> Segment<usize> {
    let end = segment.end().map(|x| x.to_usize());
    Segment::new(segment.start().to_usize(), end).unwrap()
}

struct PagesInner<T> {
    map: BTreeMap<VAddr, (Segment<VAddr>, T)>,
    buddy: Buddy,
}

pub struct Pages<T> {
    segment: Segment<VAddr>,
    inner: Mutex<PagesInner<T>>,
}

impl<T> Pages<T> {
    pub fn new(segment: Segment<VAddr>) -> Result<Pages<T>, PagesNewError> {
        use PagesNewError::*;
        if segment.is_empty() {
            return Err(ZeroSize);
        }
        Ok(Pages {
            segment,
            inner: Mutex::new(PagesInner {
                map: BTreeMap::new(),
                buddy: Buddy::new(map(segment)).unwrap(),
            }),
        })
    }
    pub fn lock(&self) -> PagesGuard<'_, T> {
        PagesGuard {
            a: self,
            g: self.inner.lock(),
        }
    }
}

pub struct PagesGuard<'a, T> {
    a: &'a Pages<T>,
    g: MutexGuard<'a, PagesInner<T>>,
}

impl<'a, T> PagesGuard<'a, T> {
    pub fn relock(&mut self) {
        unsafe {
            core::ptr::drop_in_place(&mut self.g);
            core::ptr::write(&mut self.g, self.a.inner.lock());
        }
    }
    pub fn acquire(&mut self, segment: Segment<VAddr>, t: T) -> Result<(), PagesAcquireError> {
        use PagesAcquireError::*;
        ensure!(!segment.is_empty(), ZeroSize);
        ensure!(self.a.segment.contains(segment), OutOfRange);
        let inner = &mut *self.g;
        if let Some(x) = inner.map.range(segment.start()..).next() {
            if let Some(end) = segment.end() {
                if end > *x.0 {
                    return Err(Overlapping);
                }
            }
        }
        inner.buddy.set(map(segment), true).unwrap();
        inner.map.insert(segment.start(), (segment, t));
        Ok(())
    }
    pub fn release(&mut self, vaddr: VAddr) -> Result<T, PagesReleaseError> {
        use PagesReleaseError::*;
        let inner = &mut *self.g;
        let (segment, t) = inner.map.remove(&vaddr).ok_or(NotFound)?;
        inner.buddy.set(map(segment), false).unwrap();
        Ok(t)
    }
    pub fn get(&self, vaddr: VAddr) -> Option<&T> {
        Some(&self.g.map.get(&vaddr)?.1)
    }
    pub fn locate(&self, vaddr: VAddr) -> Option<(Segment<VAddr>, &T)> {
        if let Some((_, (segment, t))) = self.g.map.range(..=vaddr).rev().next() {
            if segment.contains(vaddr) {
                return Some((*segment, t));
            }
        }
        None
    }
    pub fn find(&self, layout: MapLayout) -> Result<Segment<VAddr>, PagesFindError> {
        let inner = &*self.g;
        let vaddr = inner.buddy.find(layout.size())?;
        let vaddr = VAddr::new(vaddr);
        Ok(by_size(vaddr, layout.size()).unwrap())
    }
}
